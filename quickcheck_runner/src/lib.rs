use std::panic::{self, AssertUnwindSafe};
use serde::Deserialize;
use tonic::{transport::Server, Request, Response, Status};
use quickcheck_rpc::{
    execute_response, test_runner_server::{TestRunner, TestRunnerServer}, 
    ExecuteRequest, ExecuteResponse
};

/// A trait for test functions that can be run by the runner
pub trait TestFunction: Send + Sync + 'static {
    /// The argument type for this test function
    type Args: for<'de> Deserialize<'de> + Send + Sync + 'static;
    
    /// The return type of the test function
    type Return: serde::Serialize + Send + Sync + 'static;
    
    /// The unique name for this test function
    const TEST_ID: &'static str;
    
    /// Execute the test function with the given arguments
    fn execute(&self, args: Self::Args) -> Result<Self::Return, String>;
}

/// A runner that executes a single test function
pub struct SingleTestRunner<F: TestFunction> {
    test_function: F,
}

impl<F: TestFunction> SingleTestRunner<F> {
    pub fn new(test_function: F) -> Self {
        Self { test_function }
    }
    
    /// Start the gRPC server for this test function
    pub async fn run(self, address: &str) -> Result<(), Box<dyn std::error::Error>> {
        let addr = address.parse()?;
        
        println!("Starting gRPC Runner for '{}' on {}", F::TEST_ID, addr);

        Server::builder()
            .add_service(TestRunnerServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl<F: TestFunction> TestRunner for SingleTestRunner<F> {
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let req = request.into_inner();
        
        // Verify this is the correct test
        if req.test_id != F::TEST_ID {
            return Err(Status::not_found(format!(
                "Test '{}' not found. This runner only supports '{}'", 
                req.test_id, F::TEST_ID
            )));
        }

        // Deserialize the arguments
        let args: F::Args = rmp_serde::from_slice(&req.test_data)
            .map_err(|e| Status::invalid_argument(format!("Failed to deserialize arguments: {}", e)))?;
        
        // Execute the test function with panic catching
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            self.test_function.execute(args)
        }));

        // Convert the result to the gRPC ExecuteResponse
        let (status, failure_detail, return_value) = match result {
            Ok(Ok(return_value)) => {
                // Success case - convert return value to MessagePack
                let return_value = rmp_serde::to_vec_named(&return_value)
                    .map_err(|e| Status::internal(format!("Failed to serialize return value: {}", e)))?;
                (execute_response::TestStatus::Passed, None, Some(return_value))
            }
            Ok(Err(error_msg)) => {
                // Normal error case - return error details
                (execute_response::TestStatus::Failed, Some(error_msg), None)
            }
            Err(panic_payload) => {
                // Panic case - convert panic to error message
                let panic_msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic occurred".to_string()
                };
                (execute_response::TestStatus::Failed, Some(format!("Panic: {}", panic_msg)), None)
            }
        };

        let response = ExecuteResponse {
            status: status.into(),
            failure_detail,
            return_value,
        };

        Ok(Response::new(response))
    }
}

/// Convenience macro for creating a binary that runs a test function
#[macro_export]
macro_rules! quickcheck_runner_main {
    ($test_fn:expr, $args_ty:ty, $return_ty:ty, $test_id:expr) => {
        use quickcheck_runner::{TestFunction, SingleTestRunner};
        
        struct TestFnWrapper;
        
        impl TestFunction for TestFnWrapper {
            type Args = $args_ty;
            type Return = $return_ty;
            const TEST_ID: &'static str = $test_id;
            
            fn execute(&self, args: Self::Args) -> Result<Self::Return, String> {
                $test_fn(args)
            }
        }
        
        #[tokio::main]
        async fn main() -> Result<(), Box<dyn std::error::Error>> {
            let runner = SingleTestRunner::new(TestFnWrapper);
            runner.run("[::1]:50051").await
        }
    };
}
