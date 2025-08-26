use std::cmp;
use std::env;
use std::fmt::Debug;
use std::future::Future;
use std::panic;

// 引入 async_trait 宏
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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

// --- QuickCheck 的配置方法保持不变 ---
impl QuickCheck {
    /// Creates a new `QuickCheck` value.
    ///
    /// This can be used to run `QuickCheck` on things that implement
    /// `Testable`. You may also adjust the configuration, such as the
    /// number of tests to run.
    ///
    /// By default, the maximum number of passed tests is set to `100`, the max
    /// number of overall tests is set to `10000` and the generator is created
    /// with a size of `100`.
    pub fn new() -> QuickCheck {
        let rng = Gen::new(qc_gen_size());
        let tests = qc_tests();
        let max_tests = cmp::max(tests, qc_max_tests());
        let min_tests_passed = qc_min_tests_passed();

        QuickCheck { tests, max_tests, min_tests_passed, rng }
    }

    /// Set the random number generator to be used by `QuickCheck`.
    pub fn set_rng(self, rng: Gen) -> QuickCheck {
        QuickCheck { rng, ..self }
    }

    #[deprecated(since = "1.1.0", note = "use `set_rng` instead")]
    pub fn r#gen(self, rng: Gen) -> QuickCheck {
        self.set_rng(rng)
    }

    /// Set the number of tests to run.
    ///
    /// This actually refers to the maximum number of *passed* tests that
    /// can occur. Namely, if a test causes a failure, future testing on that
    /// property stops. Additionally, if tests are discarded, there may be
    /// fewer than `tests` passed.
    pub fn tests(mut self, tests: u64) -> QuickCheck {
        self.tests = tests;
        self
    }

    /// Set the maximum number of tests to run.
    ///
    /// The number of invocations of a property will never exceed this number.
    /// This is necessary to cap the number of tests because `QuickCheck`
    /// properties can discard tests.
    pub fn max_tests(mut self, max_tests: u64) -> QuickCheck {
        self.max_tests = max_tests;
        self
    }

    /// Set the minimum number of tests that needs to pass.
    ///
    /// This actually refers to the minimum number of *valid* *passed* tests
    /// that needs to pass for the property to be considered successful.
    pub fn min_tests_passed(mut self, min_tests_passed: u64) -> QuickCheck {
        self.min_tests_passed = min_tests_passed;
        self
    }

    // --- 主要测试方法改为 async ---

    /// (Async) Tests a property and returns the result.
    ///
    /// The result returned is either the number of tests passed or a witness
    /// of failure.
    ///
    /// (If you're using Rust's unit testing infrastructure, then you'll
    /// want to use the `quickcheck` method, which will `panic!` on failure.)
    pub async fn quicktest<A>(&mut self, f: A) -> Result<u64, TestResult>
    where
        A: Testable + Send + Sync,
    {
        let mut n_tests_passed = 0;
        for _ in 0..self.max_tests {
            if n_tests_passed >= self.tests {
                break;
            }
            // 调用异步的 result 方法并 .await
            match f.result(&mut self.rng).await {
                TestResult { status: Pass, .. } => n_tests_passed += 1,
                TestResult { status: Discard, .. } => continue,
                r @ TestResult { status: Fail, .. } => return Err(r),
            }
        }
        Ok(n_tests_passed)
    }

    /// (Async) Tests a property and calls `panic!` on failure.
    ///
    /// The `panic!` message will include a (hopefully) minimal witness of
    /// failure.
    ///
    /// It is appropriate to use this method with Rust's unit testing
    /// infrastructure.
    ///
    /// Note that if the environment variable `RUST_LOG` is set to enable
    /// `info` level log messages for the `quickcheck` crate, then this will
    /// include output on how many `QuickCheck` tests were passed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use quickcheck::QuickCheck;
    ///
    /// fn prop_reverse_reverse() {
    ///     fn revrev(xs: Vec<usize>) -> bool {
    ///         let rev: Vec<_> = xs.clone().into_iter().rev().collect();
    ///         let revrev: Vec<_> = rev.into_iter().rev().collect();
    ///         xs == revrev
    ///     }
    ///     QuickCheck::new().quickcheck(revrev as fn(Vec<usize>) -> bool);
    /// }
    /// ```
    pub async fn quickcheck<A>(&mut self, f: A)
    where
        A: Testable + Send + Sync,
    {
        // let _ = crate::env_logger_init();

        // 调用异步的 quicktest 方法并 .await
        let n_tests_passed = match self.quicktest(f).await {
            Ok(n_tests_passed) => n_tests_passed,
            Err(result) => panic!("{}", result.failed_msg()),
        };

        if n_tests_passed >= self.min_tests_passed {
            log::info!("(Passed {} QuickCheck tests.)", n_tests_passed);
        } else {
            panic!(
                "(Unable to generate enough tests, {} not discarded.)",
                n_tests_passed
            );
        }
    }
}

/// Convenience function for running `QuickCheck`.
///
/// This is an alias for `QuickCheck::new().quickcheck(f)`.
pub async fn quickcheck<A: Testable + Send + Sync>(f: A) {
    QuickCheck::new().quickcheck(f).await
}

/// Describes the status of a single instance of a test.
///
/// All testable things must be capable of producing a `TestResult`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestResult {
    status: Status,
    arguments: Option<Vec<String>>,
    err: Option<String>,
}

/// Whether a test has passed, failed or been discarded.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum Status {
    Pass,
    Fail,
    Discard,
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
        r.err = Some(msg.into());
        r
    }

    /// Produces a test result that instructs `quickcheck` to ignore it.
    /// This is useful for restricting the domain of your properties.
    /// When a test is discarded, `quickcheck` will replace it with a
    /// fresh one (up to a certain limit).
    pub fn discard() -> TestResult {
        TestResult { status: Discard, arguments: None, err: None }
    }

    /// Converts a `bool` to a `TestResult`. A `true` value indicates that
    /// the test has passed and a `false` value indicates that the test
    /// has failed.
    pub fn from_bool(b: bool) -> TestResult {
        TestResult {
            status: if b { Pass } else { Fail },
            arguments: None,
            err: None,
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
        self.is_failure() && self.err.is_some()
    }
    fn failed_msg(&self) -> String {
        let arguments_msg = match self.arguments {
            None => "No Arguments Provided".to_owned(),
            Some(ref args) => format!("Arguments: ({})", args.join(", ")),
        };
        match self.err {
            None => format!("[quickcheck] TEST FAILED. {arguments_msg}"),
            Some(ref err) => format!(
                "[quickcheck] TEST FAILED (runtime error). {arguments_msg}\nError: {err}"
            ),
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

// --- Testable Trait 和实现都改为 async ---

/// `Testable` describes types (e.g., a function) whose values can be
/// tested.
///
/// Anything that can be tested must be capable of producing a `TestResult`
/// given a random number generator. This is trivial for types like `bool`,
/// which are just converted to either a passing or failing test result.
///
/// For functions, an implementation must generate random arguments
/// and potentially shrink those arguments if they produce a failure.
///
/// It's unlikely that you'll have to implement this trait yourself.
#[async_trait]
pub trait Testable: 'static + Send + Sync {
    async fn result(&self, _: &mut Gen) -> TestResult;
}

#[async_trait]
impl Testable for bool {
    async fn result(&self, _: &mut Gen) -> TestResult {
        TestResult::from_bool(*self)
    }
}

#[async_trait]
impl Testable for () {
    async fn result(&self, _: &mut Gen) -> TestResult {
        TestResult::passed()
    }
}

#[async_trait]
impl Testable for TestResult {
    async fn result(&self, _: &mut Gen) -> TestResult {
        self.clone()
    }
}

#[async_trait]
impl<A, E> Testable for Result<A, E>
where
    A: Testable,
    E: Debug + Send + Sync + 'static,
{
    async fn result(&self, g: &mut Gen) -> TestResult {
        match *self {
            Ok(ref r) => r.result(g).await,
            Err(ref err) => TestResult::error(format!("{err:?}")),
        }
    }
}

/// Return a vector of the debug formatting of each item in `args`
fn debug_reprs(args: &[&dyn Debug]) -> Vec<String> {
    args.iter().map(|x| format!("{x:?}")).collect()
}

macro_rules! testable_fn {
    ($($name: ident),*) => {
        #[async_trait]
        impl<T, Fut, $($name),*> Testable for fn($($name),*) -> Fut
        where
            T: Testable + Send + Debug,
            Fut: Future<Output = T> + Send + 'static,
            $($name: Arbitrary + Debug + Send + Sync + Serialize + 'static),*
        {
            #[allow(non_snake_case)]
            async fn result(&self, g: &mut Gen) -> TestResult {
                async fn shrink_failure<T, Fut, $($name),*>(
                    g: &mut Gen,
                    self_: fn($($name),*) -> Fut,
                    a: ($($name,)*),
                ) -> Option<TestResult>
                where
                    T: Testable + Send + Debug,
                    Fut: Future<Output = T> + Send,// + 'static,
                    $($name: Arbitrary + Debug + Send + Sync + Serialize + 'static),*
                {
                    let shrunk_values: Vec<_> = a.shrink().collect();
                    for t in shrunk_values {
                        let ($($name,)*) = t.clone();
                        let future = self_($($name),*);
                        let testable = future.await;
                        let mut r_new = testable.result(g).await;

                        let args_json = serde_json::to_string(&t).unwrap();
                        println!("args: {:#?}, result: {:#?}", args_json, testable);

                        if r_new.is_failure() {
                            {
                                let ($(ref $name,)*) : ($($name,)*) = t;
                                r_new.arguments = Some(debug_reprs(&[$($name),*]));
                            }
                            // *** THE FIX IS HERE ***
                            // We wrap the recursive call in `Box::pin` to break the infinite type recursion.
                            let shrunk = Box::pin(shrink_failure(g, self_, t)).await;
                            
                            return Some(shrunk.unwrap_or(r_new));
                        }
                    }
                    None
                }

                let a: ($($name,)*) = Arbitrary::arbitrary(g);

                let ($($name,)*) = a.clone();
                let future = (*self)($($name),*);
                let testable = future.await;
                let r = testable.result(g).await;

                let args_json = serde_json::to_string(&a).unwrap();
                println!("args: {:#?}, result: {:#?}", args_json, testable);

                match r.status {
                    Pass | Discard => r,
                    Fail => shrink_failure(g, *self, a).await.unwrap_or(r),
                }
            }
        }
    }
}

// The `testable_fn!` macro is designed to generate `Testable` implementations
// for functions with varying numbers of arguments.
// Each invocation of the macro expands to an `impl Testable for Fun` block,
// where `Fun` is a generic function type that takes `N` arguments and returns
// a `Future<Output = T>`.
// The `async_trait` macro handles the complexities of implementing async traits.
testable_fn!();
testable_fn!(A);
testable_fn!(A, B);
testable_fn!(A, B, C);
testable_fn!(A, B, C, D);
testable_fn!(A, B, C, D, E);
testable_fn!(A, B, C, D, E, F);
testable_fn!(A, B, C, D, E, F, G);
testable_fn!(A, B, C, D, E, F, G, H);

// /// (Async) Safely executes a function and catches panics.
// async fn safe<T, F>(fun: F) -> T
// where
//     F: FnOnce() -> T + Send + 'static,
//     T: Send + 'static,
// {
//     // 将可能 panic 的同步代码移到阻塞线程池中
//     match tokio::task::spawn_blocking(move || {
//         panic::catch_unwind(panic::AssertUnwindSafe(fun))
//     })
//     .await
//     {
//         Ok(Ok(val)) => val, // 任务成功，函数也成功
//         Ok(Err(any_err)) => {
//             // 任务成功，但函数 panic
//             let err_msg = if let Some(&s) = any_err.downcast_ref::<&str>() {
//                 s.to_owned()
//             } else if let Some(s) = any_err.downcast_ref::<String>() {
//                 s.to_owned()
//             } else {
//                 "UNABLE TO SHOW RESULT OF PANIC.".to_owned()
//             };
//             // 因为 result() 返回 TestResult，我们可以构造一个 error result
//             // 这需要 T 是 TestResult，或者我们需要改变函数的签名
//             // 为了保持与原始逻辑最接近，这里我们将 panic 转换为 TestResult::error
//             // 这需要 T 能够从 TestResult::error 转换而来，这很复杂。
//             // 让我们简化一下，假设 T 是 TestResult 本身。
//             TestResult::error(err_msg)
//         }
//         Err(_) => {
//             // 任务本身失败 (e.g., runtime shutdown)
//             TestResult::error("Tokio task join error.".to_owned())
//         }
//     }
// }

// --- 单元测试改为 async 版本 ---
#[cfg(test)]
mod test {
    use super::*; // 导入父模块的所有内容

    #[tokio::test]
    async fn shrinking_regression_issue_126() {
        async fn the_test(vals: Vec<bool>) -> bool {
            vals.iter().filter(|&v| *v).count() < 2
        }

        let failing_case = QuickCheck::new()
            .quicktest(the_test as fn(Vec<bool>) -> _)
            .await
            .unwrap_err();
        
        // Note: The shrunk value might be `[true, true]`. A vec formats with brackets.
        let expected_argument = format!("{:?}", vec![true, true]);
        assert_eq!(failing_case.arguments, Some(vec![expected_argument]));
    }

    #[tokio::test]
    async fn size_for_small_types_issue_143() {
        async fn t(_: i8) -> bool {
            true
        }
        QuickCheck::new().set_rng(Gen::new(129)).quickcheck(t as fn(i8) -> _).await;
    }

    #[tokio::test]
    async fn regression_signed_shrinker_panic() {
        async fn foo_can_shrink(v: i8) -> bool {
            let _ = crate::Arbitrary::shrink(&v).take(100).count();
            true
        }
        quickcheck(foo_can_shrink as fn(i8) -> _).await;
    }
}
