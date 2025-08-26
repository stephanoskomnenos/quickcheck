use std::collections::HashMap;
use serde::Deserialize;
use tonic::{transport::Server, Request, Response, Status};
use serde_json::Value;

// Assuming your crate is named `your_crate_name`
// This uses the generated code from your quickcheck_rpc lib.rs
use quickcheck_rpc::{
    execute_response, test_runner_server::{TestRunner, TestRunnerServer}, ExecuteRequest, ExecuteResponse
};
use quickcheck::TestResult; // Use your internal TestResult

// Define a type for our dispatchable property functions
type TestFn = fn(Value) -> TestResult;

// --- Your actual test properties live here ---
// They take a `serde_json::Value` and deserialize their own arguments.

#[derive(Deserialize, Debug, Clone)]
struct ReverseArgs {
    xs: Vec<usize>,
}
fn reverse_test(args_value: Value) -> TestResult {
    let args: ReverseArgs = serde_json::from_value(args_value)
        .expect("Runner: failed to deserialize 'reverse' arguments");
    let xs = args.xs;

    let rev: Vec<_> = xs.clone().into_iter().rev().collect();
    let revrev: Vec<_> = rev.into_iter().rev().collect();
    TestResult::from_bool(xs == revrev)
}

#[derive(Deserialize, Debug, Clone)]
struct AddArgs {
    a: i32,
    b: i32,
}

fn add_test(args_value: Value) -> TestResult {
    let args: AddArgs = serde_json::from_value(args_value)
        .expect("Runner: failed to deserialize 'add' arguments");
    let a = args.a;
    let b = args.b;

    let result = a + b;
    println!("{:?} + {:?} = {:?}", a, b, result);
    TestResult::from_bool(result == a + b)
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
        
        // Execute the property
        let internal_result = property_fn(args);

        // Convert the internal TestResult to the gRPC ExecuteResponse
        let response = ExecuteResponse {
            status: if !internal_result.is_failure() {
                execute_response::TestStatus::Passed.into()
            } else {
                execute_response::TestStatus::Failed.into()
            },
            failure_detail: None,
            // failure_detail: internal_result.err,
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