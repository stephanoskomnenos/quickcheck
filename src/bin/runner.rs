use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use serde::Deserialize;
use tonic::{transport::Server, Request, Response, Status};
use serde_json::Value;

// Assuming your crate is named `your_crate_name`
// This uses the generated code from your quickcheck_rpc lib.rs
use quickcheck_rpc::{
    execute_response, test_runner_server::{TestRunner, TestRunnerServer}, ExecuteRequest, ExecuteResponse
};

// Define a type for our dispatchable property functions that return a value
type TestFn = fn(Value) -> Result<Value, String>;

// --- Your actual test properties live here ---
// They take a `serde_json::Value` and deserialize their own arguments.

#[derive(Deserialize, Debug, Clone)]
struct ReverseArgs {
    xs: Vec<usize>,
}
fn reverse_test(args_value: Value) -> Result<Value, String> {
    let args: ReverseArgs = serde_json::from_value(args_value)
        .map_err(|e| format!("Runner: failed to deserialize 'reverse' arguments: {}", e))?;
    let xs = args.xs;

    let rev: Vec<_> = xs.clone().into_iter().rev().collect();
    let revrev: Vec<_> = rev.into_iter().rev().collect();
    
    // Return the reversed result for comparison in composite tests
    serde_json::to_value(revrev).map_err(|e| e.to_string())
}

#[derive(Deserialize, Debug, Clone)]
struct AddArgs {
    a: i32,
    b: i32,
}

fn add_test(args_value: Value) -> Result<Value, String> {
    let args: AddArgs = serde_json::from_value(args_value)
        .map_err(|e| format!("Runner: failed to deserialize 'add' arguments: {}", e))?;
    let a = args.a;
    let b = args.b;

    let result = a + b;
    println!("{:?} + {:?} = {:?}", a, b, result);
    
    // Return the actual result for comparison in composite tests
    serde_json::to_value(result).map_err(|e| e.to_string())
}

// A test function that panics to verify panic handling
fn panic_test(_args_value: Value) -> Result<Value, String> {
    panic!("This is a test panic to verify panic handling");
}
// The gRPC service implementation
#[derive(Default)]
pub struct MyTestRunner {
    tests: HashMap<String, TestFn>,
}

impl MyTestRunner {
    fn new() -> Self {
        let mut tests = HashMap::new();
        // Register the properties the runner knows how to execute
        tests.insert("property_reverse_list".to_string(), reverse_test as TestFn);
        tests.insert("property_add".to_string(), add_test as TestFn);
        tests.insert("property_panic".to_string(), panic_test as TestFn);
        Self { tests }
    }
}

#[tonic::async_trait]
impl TestRunner for MyTestRunner {
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let req = request.into_inner();
        println!("Runner: Received request for property '{}'", req.property_name);

        // Find the correct property function to run
        let property_fn = self.tests.get(&req.property_name).ok_or_else(|| {
            Status::not_found(format!("Property '{}' not found", req.property_name))
        })?;

        // Deserialize the arguments from the JSON string
        let args: Value = serde_json::from_str(&req.test_data_json)
            .map_err(|e| Status::invalid_argument(format!("Failed to parse JSON: {}", e)))?;
        
        // Execute the property with panic catching
        let result = panic::catch_unwind(AssertUnwindSafe(|| property_fn(args)));

        // Convert the result to the gRPC ExecuteResponse
        let (status, failure_detail, return_value_json) = match result {
            Ok(Ok(return_value)) => {
                // Success case - convert return value to JSON string
                let return_json = serde_json::to_string(&return_value)
                    .map_err(|e| Status::internal(format!("Failed to serialize return value: {}", e)))?;
                (execute_response::TestStatus::Passed, None, Some(return_json))
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
            failure_detail: failure_detail,
            return_value_json: return_value_json,
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let runner = MyTestRunner::new();

    println!("gRPC Runner listening on {}", addr);

    Server::builder()
        .add_service(TestRunnerServer::new(runner))
        .serve(addr)
        .await?;

    Ok(())
}
