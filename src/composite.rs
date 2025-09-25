use async_trait::async_trait;
use std::fmt::Debug;

use crate::{Arbitrary, Gen, TestResult};

/// A composite property that compares results from multiple Property implementations
pub struct CompositeProperty<P1, P2, F>
where
    P1: crate::tester::Property + Send + Sync,
    P2: crate::tester::Property + Send + Sync,
    F: Fn(&P1::Args, P1::Return, P2::Return) -> bool + Send + Sync + 'static,
{
    prop1: P1,
    prop2: P2,
    comparison: F,
}

impl<P1, P2, F> CompositeProperty<P1, P2, F>
where
    P1: crate::tester::Property + Send + Sync,
    P2: crate::tester::Property + Send + Sync,
    F: Fn(&P1::Args, P1::Return, P2::Return) -> bool + Send + Sync + 'static,
{
    pub fn new(prop1: P1, prop2: P2, comparison: F) -> Self {
        Self {
            prop1,
            prop2,
            comparison,
        }
    }
}

#[async_trait]
impl<P1, P2, F> crate::tester::Testable for CompositeProperty<P1, P2, F>
where
    P1: crate::tester::Property + Send + Sync + 'static,
    P2: crate::tester::Property + Send + Sync + 'static,
    P1::Args: Arbitrary + Debug + Clone + Send + Sync,
    P2::Args: From<P1::Args>,
    F: Fn(&P1::Args, P1::Return, P2::Return) -> bool + Send + Sync + 'static,
{
    async fn result(&self, g: &mut Gen) -> TestResult {
        // Generate arguments for the first property
        let args: P1::Args = Arbitrary::arbitrary(g);
        
        // Execute both properties
        let result1 = execute_property(&self.prop1, &args).await;
        let args2: P2::Args = args.clone().into();
        let result2 = execute_property(&self.prop2, &args2).await;
        
        // Check if both executions were successful
        if result1.is_failure() || result2.is_failure() {
            return TestResult {
                status: crate::tester::Status::Fail,
                arguments: vec![format!("{:?}", args)],
                err: Some(format!(
                    "Property execution failed: prop1={:?}, prop2={:?}",
                    result1.err, result2.err
                )),
                return_value: None,
            };
        }
        
        // Extract return values
        let return1 = extract_return_value::<P1>(&result1);
        let return2 = extract_return_value::<P2>(&result2);
        
        match (return1, return2) {
            (Ok(val1), Ok(val2)) => {
                // Compare the results using the provided comparison function
                if (self.comparison)(&args, val1, val2) {
                    TestResult::passed()
                } else {
                    TestResult {
                        status: crate::tester::Status::Fail,
                        arguments: vec![format!("{:?}", args)],
                        err: Some("Comparison function returned false".to_string()),
                        return_value: None,
                    }
                }
            }
            (Err(e), _) | (_, Err(e)) => TestResult {
                status: crate::tester::Status::Fail,
                arguments: vec![format!("{:?}", args)],
                err: Some(format!("Failed to extract return values: {}", e)),
                return_value: None,
            },
        }
    }
}

/// Helper function to execute a property and return its result
async fn execute_property<P: crate::tester::Property + 'static>(
    prop: &P,
    args: &P::Args,
) -> TestResult {
    use crate::tester::Testable;
    
    // Create a new generator for each execution to ensure consistent behavior
    let mut g = Gen::new(100);
    prop.result(&mut g).await
}

/// Helper function to extract the return value from a TestResult
fn extract_return_value<P: crate::tester::Property>(result: &TestResult) -> Result<P::Return, String> {
    if let Some(ref json_str) = result.return_value {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to deserialize return value: {}", e))
    } else {
        Err("No return value available".to_string())
    }
}

/// Macro for creating composite tests
#[macro_export]
macro_rules! quickcheck_composite {
    ($prop1:expr, $prop2:expr, |$args:ident, $res1:ident, $res2:ident| $comparison:expr) => {
        $crate::composite::CompositeProperty::new($prop1, $prop2, |$args, $res1, $res2| $comparison)
    };
}
