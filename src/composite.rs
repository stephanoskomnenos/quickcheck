use async_trait::async_trait;
use std::fmt::Debug;

use crate::{Arbitrary, TestResult};

/// A composite property that compares results from multiple Property implementations
pub struct CompositeProperty<P, F>
where
    P: crate::tester::Property + Send + Sync,
    F: Fn(&P::Args, &[P::Return]) -> bool + Send + Sync + 'static,
{
    props: Vec<P>,
    comparison: F,
}

impl<P, F> CompositeProperty<P, F>
where
    P: crate::tester::Property + Send + Sync,
    F: Fn(&P::Args, &[P::Return]) -> bool + Send + Sync + 'static,
{
    pub fn new(props: Vec<P>, comparison: F) -> Self {
        Self {
            props,
            comparison,
        }
    }
}

#[async_trait]
impl<P, F> crate::tester::Testable for CompositeProperty<P, F>
where
    P: crate::tester::Property + Send + Sync + 'static,
    P::Args: Arbitrary + Debug + Clone + Send + Sync,
    F: Fn(&P::Args, &[P::Return]) -> bool + Send + Sync + 'static,
{
    type Args = P::Args;
    
    async fn result(&self, args: &Self::Args) -> TestResult {
        async fn execute_properties<P: crate::tester::Property + 'static>(
            props: &[P],
            args: &P::Args,
        ) -> Result<Vec<P::Return>, TestResult> {
            /// Helper function to extract the return value from a TestResult
            fn extract_return_value<P: crate::tester::Property>(result: &TestResult) -> Result<P::Return, String> {
                if let Some(ref msgpack) = result.return_value {
                    rmp_serde::from_slice(msgpack)
                        .map_err(|e| format!("Failed to deserialize return value: {}", e))
                } else {
                    Err("No return value available".to_string())
                }
            }

            let mut results = Vec::new();
            for prop in props {
                let result = prop.result(args).await;
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
                match extract_return_value::<P>(result) {
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

        async fn shrink_failure<P, F>( // Correctly placed inside result
            composite: &CompositeProperty<P, F>,
            initial_args: P::Args,
        ) -> Option<TestResult>
        where
            P: crate::tester::Property + Send + Sync + 'static,
            P::Args: Arbitrary + Debug + Clone + Send + Sync,
            F: Fn(&P::Args, &[P::Return]) -> bool + Send + Sync + 'static,
        {
            println!("Shrinking composite test... Args: {:?}", initial_args);
            let shrunk_values: Vec<_> = initial_args.shrink().collect();
            
            for shrunk_args in shrunk_values {
                match execute_properties(&composite.props, &shrunk_args).await {
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

        match execute_properties(&self.props, args).await {
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

/// Macro for creating composite tests with arbitrary number of properties
#[macro_export]
macro_rules! quickcheck_composite {
    // Base case: single property (though not very useful for comparison)
    ($prop:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$prop], |$args, $results| $comparison)).await
    };
    
    // Two properties (backward compatibility)
    ($prop1:expr, $prop2:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$prop1, $prop2], |$args, $results| $comparison)).await
    };
    
    // Three properties
    ($prop1:expr, $prop2:expr, $prop3:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$prop1, $prop2, $prop3], |$args, $results| $comparison)).await
    };
    
    // Four properties
    ($prop1:expr, $prop2:expr, $prop3:expr, $prop4:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$prop1, $prop2, $prop3, $prop4], |$args, $results| $comparison)).await
    };
    
    // Five properties
    ($prop1:expr, $prop2:expr, $prop3:expr, $prop4:expr, $prop5:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$prop1, $prop2, $prop3, $prop4, $prop5], |$args, $results| $comparison)).await
    };
    
    // Six properties
    ($prop1:expr, $prop2:expr, $prop3:expr, $prop4:expr, $prop5:expr, $prop6:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$prop1, $prop2, $prop3, $prop4, $prop5, $prop6], |$args, $results| $comparison)).await
    };
    
    // Seven properties
    ($prop1:expr, $prop2:expr, $prop3:expr, $prop4:expr, $prop5:expr, $prop6:expr, $prop7:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$prop1, $prop2, $prop3, $prop4, $prop5, $prop6, $prop7], |$args, $results| $comparison)).await
    };
    
    // Eight properties
    ($prop1:expr, $prop2:expr, $prop3:expr, $prop4:expr, $prop5:expr, $prop6:expr, $prop7:expr, $prop8:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$prop1, $prop2, $prop3, $prop4, $prop5, $prop6, $prop7, $prop8], |$args, $results| $comparison)).await
    };
    
    // Nine properties
    ($prop1:expr, $prop2:expr, $prop3:expr, $prop4:expr, $prop5:expr, $prop6:expr, $prop7:expr, $prop8:expr, $prop9:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$prop1, $prop2, $prop3, $prop4, $prop5, $prop6, $prop7, $prop8, $prop9], |$args, $results| $comparison)).await
    };
    
    // Ten properties (maximum practical limit)
    ($prop1:expr, $prop2:expr, $prop3:expr, $prop4:expr, $prop5:expr, $prop6:expr, $prop7:expr, $prop8:expr, $prop9:expr, $prop10:expr, |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$prop1, $prop2, $prop3, $prop4, $prop5, $prop6, $prop7, $prop8, $prop9, $prop10], |$args, $results| $comparison)).await
    };
    
    // Variadic version using recursion (supports any number of properties)
    ($($props:expr),+ , |$args:ident, $results:ident| $comparison:expr) => {
        $crate::quickcheck($crate::CompositeProperty::new(vec![$($props),+], |$args, $results| $comparison)).await
    };
}