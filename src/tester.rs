use std::env;
use std::fmt::Debug;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Use the gRPC client types from the quickcheck_rpc crate.
use quickcheck_rpc::{
    execute_response::TestStatus as ProtoStatus,
    test_runner_client::TestRunnerClient, ExecuteRequest,
};

use crate::{
    tester::Status::{Discard, Fail, Pass},
    Arbitrary, Gen,
};

/// The main `QuickCheck` type for setting configuration and running `QuickCheck`. 
pub struct QuickCheck {
    tests: u64,
    max_tests: u64,
    min_tests_passed: u64,
    rng: Gen,
}

// --- 配置函数 (qc_*) 保持不变 ---
fn qc_tests() -> u64 {
    let default = 100;
    match env::var("QUICKCHECK_TESTS") {
        Ok(val) => val.parse().unwrap_or(default),
        Err(_) => default,
    }
}

fn qc_max_tests() -> u64 {
    let default = 10_000;
    match env::var("QUICKCHECK_MAX_TESTS") {
        Ok(val) => val.parse().unwrap_or(default),
        Err(_) => default,
    }
}

fn qc_gen_size() -> usize {
    let default = 100;
    match env::var("QUICKCHECK_GENERATOR_SIZE") {
        Ok(val) => val.parse().unwrap_or(default),
        Err(_) => default,
    }
}

fn qc_min_tests_passed() -> u64 {
    let default = 0;
    match env::var("QUICKCHECK_MIN_TESTS_PASSED") {
        Ok(val) => val.parse().unwrap_or(default),
        Err(_) => default,
    }
}

impl Default for QuickCheck {
    fn default() -> Self {
        Self::new()
    }
}

// (Full implementation for QuickCheck and its methods is omitted for brevity)
#[allow(dead_code)]
impl QuickCheck {
    pub fn new() -> Self {
        Self {
            tests: 100,
            max_tests: 10000,
            min_tests_passed: 0,
            rng: Gen::new(100),
        }
    }
    pub async fn quicktest<A>(&mut self, f: A) -> Result<u64, TestResult>
    where
        A: Testable,
    {
        let mut n_tests_passed = 0;
        for _ in 0..self.max_tests {
            if n_tests_passed >= self.tests {
                break;
            }
            let args = A::Args::arbitrary(&mut self.rng);
            match f.result(&args).await {
                TestResult { status: Pass, .. } => n_tests_passed += 1,
                TestResult { status: Discard, .. } => (),
                r @ TestResult { status: Fail, .. } => return Err(r),
            }
        }
        Ok(n_tests_passed)
    }
    pub async fn quickcheck<A>(&mut self, f: A)
    where
        A: Testable,
    {
        // Ignore log init failures, implying it has already been done.
        let _ = crate::env_logger_init();

        let n_tests_passed = match self.quicktest(f).await {
            Ok(n_tests_passed) => n_tests_passed,
            Err(result) => {
                if result.is_error() { // is_error() checks for TestFailure::Runtime
                    // It's a runtime error, so we want the stack trace.
                    panic!("{}", result.failed_msg());
                } else {
                    // It's a property or comparison failure, so exit cleanly.
                    eprintln!("{}", result.failed_msg());
                    std::process::exit(1);
                }
            }
        };

        if n_tests_passed >= self.min_tests_passed {
            info!("(Passed {} QuickCheck tests.)", n_tests_passed);
        } else {
            panic!(
                "(Unable to generate enough tests, {} not discarded.)",
                n_tests_passed
            );
        }
    }
}
pub async fn quickcheck<A: Testable + Send + Sync>(a: A) {
    QuickCheck::new().quickcheck(a).await;
}

// --- TestResult and Status types are kept for reporting ---
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TestFailure {
    Property(Option<String>), // Detail from runner
    Comparison,               // No extra detail needed, or a default message
    Runtime(String),          // Detail of the runtime error
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct TestResult {
    pub status: Status,
    pub arguments: Vec<String>,
    #[serde(default)] // Ensure default is handled for deserialization
    pub failure: Option<TestFailure>, // New field, replaces `err` and `err_type`
    pub return_value: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub enum Status {
    #[default]
    Pass,
    Fail,
    Discard,
}

impl From<ProtoStatus> for Status {
    fn from(s: ProtoStatus) -> Self {
        match s {
            ProtoStatus::Passed => Pass,
            ProtoStatus::Failed => Fail,
            ProtoStatus::InvalidInput => Discard,
        }
    }
}

impl TestResult {
    /// Produces a test result that indicates the current test has passed.
    pub fn passed() -> TestResult {
        TestResult::from_bool(true)
    }

    /// Produces a test result that indicates the current test has failed.
    pub fn failed() -> TestResult {
        TestResult::from_bool(false)
    }

    /// Produces a test result that indicates failure from a runtime error.
    pub fn error<S: Into<String>>(msg: S) -> TestResult {
        let mut r = TestResult::from_bool(false);
        r.failure = Some(TestFailure::Runtime(msg.into()));
        r
    }

    /// Produces a test result that instructs `quickcheck` to ignore it.
    /// This is useful for restricting the domain of your properties.
    /// When a test is discarded, `quickcheck` will replace it with a
    /// fresh one (up to a certain limit).
    pub fn discard() -> TestResult {
        TestResult {
            status: Discard,
            arguments: vec![],
            failure: None,
            return_value: None,
        }
    }

    /// Converts a `bool` to a `TestResult`. A `true` value indicates that
    /// the test has passed and a `false` value indicates that the test
    /// has failed.
    pub fn from_bool(b: bool) -> TestResult {
        TestResult {
            status: if b { Pass } else { Fail },
            arguments: vec![],
            failure: if b { None } else { Some(TestFailure::Property(None)) },
            return_value: None,
        }
    }

    // must_fail 需要用 spawn_blocking 改造，因为它依赖 panic::catch_unwind
    // 为简化，此处暂时注释，实际项目中需要异步化改造
    /// Tests if a "procedure" fails when executed. The test passes only if
    /// `f` generates a task failure during its execution.
    /*
    pub fn must_fail<T, F>(f: F) -> TestResult
    where
        F: FnOnce() -> T,
        F: 'static,
        T: 'static,
    {
        let f = panic::AssertUnwindSafe(f);
        TestResult::from_bool(panic::catch_unwind(f).is_err())
    }
    */

    /// Returns `true` if and only if this test result describes a failing
    /// test.
    pub fn is_failure(&self) -> bool {
        matches!(self.status, Fail)
    }

    /// Returns `true` if and only if this test result describes a failing
    /// test as a result of a run time error.
    pub fn is_error(&self) -> bool {
        matches!(self.failure, Some(TestFailure::Runtime(_)))
    }
    fn failed_msg(&self) -> String {
        let arguments_msg = format!("Arguments: ({})", self.arguments.join(", "));
        match &self.failure {
            Some(TestFailure::Runtime(err_msg)) => format!(
                "[quickcheck] TEST FAILED (runtime error). {arguments_msg}\nError: {err_msg}"
            ),
            Some(TestFailure::Property(Some(err_msg))) => format!(
                "[quickcheck] TEST FAILED. {arguments_msg}\nError: {err_msg}"
            ),
            Some(TestFailure::Property(None)) => format!(
                "[quickcheck] TEST FAILED. {arguments_msg}"
            ),
            Some(TestFailure::Comparison) => format!(
                "[quickcheck] TEST FAILED (comparison). {arguments_msg}\nError: Comparison function returned false"
            ),
            None => format!("[quickcheck] TEST PASSED. {arguments_msg}"), // Should not happen if status is Fail
        }
    }
}

impl From<bool> for TestResult {
    /// A shorter way of producing a `TestResult` from a `bool`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use quickcheck::TestResult;
    /// let result: TestResult = (2 > 1).into();
    /// assert_eq!(result, TestResult::passed());
    /// ```
    fn from(b: bool) -> TestResult {
        TestResult::from_bool(b)
    }
}

// --- The New `Property` Trait and its `Testable` Implementation ---

/// `Testable` is the central trait that `QuickCheck` uses.
#[async_trait]
pub trait Testable: 'static + Send + Sync {
    /// The argument type for this testable
    type Args: Arbitrary + Debug + Clone + Send + Sync + 'static;
    
    async fn result(&self, args: &Self::Args) -> TestResult;
}

/// A new trait to define a remote property and its argument structure.
pub trait Property: Send + Sync {
    /// The struct that holds the arguments for this property.
    type Args: Arbitrary + Serialize + Debug + Clone + Send + Sync + 'static;

    /// The return type of the property function, which must be deserializable.
    type Return: for<'de> Deserialize<'de>
        + Debug
        + Clone
        + Send
        + Sync
        + 'static;

    /// The unique string ID for this property, matching the ID in the runner.
    const PROPERTY_NAME: &'static str;

    /// The network address of the gRPC runner server.
    fn endpoint(&self) -> &str;
}

/// Implements `Testable` for any type that implements our new `Property` trait.
#[async_trait]
impl<P> Testable for P
where
    P: Property + 'static,
{
    type Args = P::Args;
    
    async fn result(&self, args: &Self::Args) -> TestResult {
        async fn execute_remote<Pr: Property>(
            prop: &Pr,
            args: &Pr::Args,
        ) -> Result<TestResult, String> {
            let mut client =
                TestRunnerClient::connect(prop.endpoint().to_string())
                    .await
                    .map_err(|e| e.to_string())?;
            let args_msgpack =
                rmp_serde::to_vec_named(args).map_err(|e| e.to_string())?;
            // println!("args_json: {:#?}", args_json);
            let request = tonic::Request::new(ExecuteRequest {
                property_name: Pr::PROPERTY_NAME.to_string(),
                test_data: args_msgpack,
            });
            let response = client
                .execute(request)
                .await
                .map_err(|e| e.to_string())?
                .into_inner();
            // println!("response: {:#?}", response);
            let proto_status = ProtoStatus::try_from(response.status)
                .unwrap_or(ProtoStatus::Failed);
            Ok(TestResult {
                status: proto_status.into(),
                arguments: vec![format!("{:?}", args)],
                failure: if proto_status == ProtoStatus::Failed {
                    Some(TestFailure::Property(response.failure_detail))
                } else { None },
                return_value: response.return_value,
            })
        }

        async fn shrink_failure<Pr: Property>(
            prop: &Pr,
            initial_args: Pr::Args,
        ) -> Option<TestResult> {
            println!("Shrinking... Args: {:?}", initial_args);
            // Collect the iterator into a Vec to hold across await points
            let shrunk_values: Vec<_> = initial_args.shrink().collect();
            
            for shrunk_args in shrunk_values {
                match execute_remote(prop, &shrunk_args).await {
                    Ok(new_result) => {
                        if new_result.is_failure() {
                            let smaller_failure = Box::pin(shrink_failure(prop, shrunk_args)).await;
                            
                            if let Some(smaller_result) = smaller_failure {
                                return Some(smaller_result);
                            } else {
                                return Some(new_result);
                            }
                        }
                    }
                    Err(e) => {
                        // A runtime error occurred during shrinking.
                        // This is a candidate for the smallest failure.
                        return Some(TestResult {
                            status: Fail,
                            arguments: vec![format!("{:?}", shrunk_args)],
                            failure: Some(TestFailure::Runtime(format!("Tester failed to call runner: {}", e))),
                            return_value: None,
                        });
                    }
                }
            }
            None
        }

        match execute_remote(self, args).await {
            Ok(result) => {
                if result.is_failure() {
                    // Start shrink process for failing test
                    shrink_failure(self, args.clone()).await.unwrap_or(result)
                } else {
                    result
                }
            }
            Err(e) => TestResult {
                status: Fail,
                arguments: vec![format!("{:?}", args)],
                failure: Some(TestFailure::Runtime(format!("Tester failed to call runner: {}", e))),
                return_value: None,
            },
        }
    }
}
