use async_trait::async_trait;
use std::fmt::Debug;

use crate::{Arbitrary, TestResult};

/// A composite test that compares results from multiple RemoteTest implementations
pub struct CompositeTest<T, F>
where
    T: crate::tester::RemoteTest + Send + Sync,
    F: Fn(&T::Args, &[T::Return]) -> bool + Send + Sync + 'static,
{
    tests: Vec<T>,
    comparison: F,
}

impl<T, F> CompositeTest<T, F>
where
    T: crate::tester::RemoteTest + Send + Sync,
    F: Fn(&T::Args, &[T::Return]) -> bool + Send + Sync + 'static,
{
    pub fn new(tests: Vec<T>, comparison: F) -> Self {
        Self {
            tests,
            comparison,
        }
    }
}

#[async_trait]
impl<T, F> crate::tester::Testable for CompositeTest<T, F>
where
    T: crate::tester::RemoteTest + Send + Sync + 'static,
    T::Args: Arbitrary + Debug + Clone + Send + Sync,
    F: Fn(&T::Args, &[T::Return]) -> bool + Send + Sync + 'static,
{
    type Args = T::Args;
    
    async fn result(&self, args: &Self::Args) -> TestResult {
        async fn execute_tests<T: crate::tester::RemoteTest + 'static>(
            tests: &[T],
            args: &T::Args,
        ) -> Result<Vec<T::Return>, TestResult> {
            /// Helper function to extract the return value from a TestResult
            fn extract_return_value<T: crate::tester::RemoteTest>(result: &TestResult) -> Result<T::Return, String> {
                if let Some(ref msgpack) = result.return_value {
                    rmp_serde::from_slice(msgpack)
                        .map_err(|e| format!("Failed to deserialize return value: {}", e))
                } else {
                    Err("No return value available".to_string())
                }
            }

            let mut results = Vec::new();
            for test in tests {
                let result = test.result(args).await;
                if result.is_failure() {
                    return Err(TestResult {
                        status: crate::tester::Status::Fail,
                        arguments: vec![format!("{:?}", args)],
                        failure: result.failure, // Propagate the failure
                        return_value: None,
                    });
                }
                results.push(result);
            }
            
            let mut return_values = Vec::new();
            for result in &results {
                match extract_return_value::<T>(result) {
                    Ok(value) => return_values.push(value),
                    Err(e) => {
                        return Err(TestResult {
                            status: crate::tester::Status::Fail,
                            arguments: vec![format!("{:?}", args)],
                            failure: Some(crate::tester::TestFailure::Runtime(format!("Failed to extract return values: {}", e))),
                            return_value: None,
                        });
                    }
                }
            }
            Ok(return_values)
        }

        async fn shrink_failure<T, F>( // Correctly placed inside result
            composite: &CompositeTest<T, F>,
            initial_args: T::Args,
        ) -> Option<TestResult>
        where
            T: crate::tester::RemoteTest + Send + Sync + 'static,
            T::Args: Arbitrary + Debug + Clone + Send + Sync,
            F: Fn(&T::Args, &[T::Return]) -> bool + Send + Sync + 'static,
        {
            println!("Shrinking composite test... Args: {:?}", initial_args);
            let shrunk_values: Vec<_> = initial_args.shrink().collect();
            
            for shrunk_args in shrunk_values {
                match execute_tests(&composite.tests, &shrunk_args).await {
                    Ok(return_values) => {
                        if !(composite.comparison)(&shrunk_args, &return_values) {
                            // This is a smaller failing case due to comparison.
                            // Recurse to see if we can find an even smaller one.
                            let smaller_failure = Box::pin(shrink_failure(composite, shrunk_args.clone())).await;
                            if let Some(smaller_result) = smaller_failure {
                                return Some(smaller_result);
                            } else {
                                // This is the smallest we could find.
                                return Some(TestResult {
                                    status: crate::tester::Status::Fail,
                                    arguments: vec![format!("{:?}", shrunk_args)],
                                    failure: Some(crate::tester::TestFailure::Comparison),
                                    return_value: None,
                                });
                            }
                        }
                    }
                    Err(err_result) => {
                        // A non-comparison failure occurred during shrinking (e.g., runtime error).
                        // This is a candidate for the smallest failure.
                        // We should not continue shrinking, as the nature of the failure has changed.
                        // We return this error immediately as the new smallest failure.
                        return Some(err_result);
                    }
                }
            }
            None
        }

        match execute_tests(&self.tests, args).await {
            Ok(return_values) => {
                if (self.comparison)(args, &return_values) {
                    TestResult::passed()
                } else {
                    // Start shrink process for failing composite test
                    shrink_failure(self, args.clone()).await.unwrap_or_else(|| TestResult {
                        status: crate::tester::Status::Fail,
                        arguments: vec![format!("{:?}", args)],
                        failure: Some(crate::tester::TestFailure::Comparison),
                        return_value: None,
                    })
                }
            }
            Err(result) => result,
        }
    }
}

/// Macro for creating composite tests with arbitrary number of tests
#[macro_export]
macro_rules! quickcheck_composite {
    // Base case: single test (though not very useful for comparison)
    ($test:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$test], |$args, $results| $comparison)).await
    };
    
    // Two tests (backward compatibility)
    ($test1:expr, $test2:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$test1, $test2], |$args, $results| $comparison)).await
    };
    
    // Three tests
    ($test1:expr, $test2:expr, $test3:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$test1, $test2, $test3], |$args, $results| $comparison)).await
    };
    
    // Four tests
    ($test1:expr, $test2:expr, $test3:expr, $test4:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$test1, $test2, $test3, $test4], |$args, $results| $comparison)).await
    };
    
    // Five tests
    ($test1:expr, $test2:expr, $test3:expr, $test4:expr, $test5:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$test1, $test2, $test3, $test4, $test5], |$args, $results| $comparison)).await
    };
    
    // Six tests
    ($test1:expr, $test2:expr, $test3:expr, $test4:expr, $test5:expr, $test6:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$test1, $test2, $test3, $test4, $test5, $test6], |$args, $results| $comparison)).await
    };
    
    // Seven tests
    ($test1:expr, $test2:expr, $test3:expr, $test4:expr, $test5:expr, $test6:expr, $test7:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$test1, $test2, $test3, $test4, $test5, $test6, $test7], |$args, $results| $comparison)).await
    };
    
    // Eight tests
    ($test1:expr, $test2:expr, $test3:expr, $test4:expr, $test5:expr, $test6:expr, $test7:expr, $test8:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$test1, $test2, $test3, $test4, $test5, $test6, $test7, $test8], |$args, $results| $comparison)).await
    };
    
    // Nine tests
    ($test1:expr, $test2:expr, $test3:expr, $test4:expr, $test5:expr, $test6:expr, $test7:expr, $test8:expr, $test9:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$test1, $test2, $test3, $test4, $test5, $test6, $test7, $test8, $test9], |$args, $results| $comparison)).await
    };
    
    // Ten tests (maximum practical limit)
    ($test1:expr, $test2:expr, $test3:expr, $test4:expr, $test5:expr, $test6:expr, $test7:expr, $test8:expr, $test9:expr, $test10:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$test1, $test2, $test3, $test4, $test5, $test6, $test7, $test8, $test9, $test10], |$args, $results| $comparison)).await
    };
    
    // Variadic version using recursion (supports any number of tests)
    ($($tests:expr),+ , |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeTest::new(vec![$($tests),+], |$args, $results| $comparison)).await
    };
}
