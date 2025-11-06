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
            let mut results = Vec::new();
            for prop in props {
                let result = prop.result(args).await;
                if result.is_failure() {
                    return Err(TestResult {
                        status: crate::tester::Status::Fail,
                        arguments: vec![format!("{:?}", args)],
                        err: Some(format!(
                            "Property execution failed: {}",
                            result.err.unwrap_or_else(|| "Unknown error".to_string())
                        )),
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
                            err: Some(format!("Failed to extract return values: {}", e)),
                            return_value: None,
                        });
                    }
                }
            }
            Ok(return_values)
        }

        async fn shrink_failure<P, F>(
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
                if let Ok(return_values) = execute_properties(&composite.props, &shrunk_args).await {
                    if !(composite.comparison)(&shrunk_args, &return_values) {
                        let smaller_failure = Box::pin(shrink_failure(composite, shrunk_args.clone())).await;
                        if let Some(smaller_result) = smaller_failure {
                            return Some(smaller_result);
                        } else {
                            return Some(TestResult {
                                status: crate::tester::Status::Fail,
                                arguments: vec![format!("{:?}", shrunk_args)],
                                err: Some("Comparison function returned false".to_string()),
                                return_value: None,
                            });
                        }
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
                        err: Some("Comparison function returned false".to_string()),
                        return_value: None,
                    })
                }
            }
            Err(result) => result,
        }
    }
}

/// Helper function to extract the return value from a TestResult
fn extract_return_value<P: crate::tester::Property>(result: &TestResult) -> Result<P::Return, String> {
    if let Some(ref msgpack) = result.return_value {
        rmp_serde::from_slice(msgpack)
            .map_err(|e| format!("Failed to deserialize return value: {}", e))
    } else {
        Err("No return value available".to_string())
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
