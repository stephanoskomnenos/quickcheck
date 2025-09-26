use quickcheck_macros::Arbitrary;
use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck_composite, Property};

#[derive(Serialize, Deserialize, Debug, Clone, Arbitrary)]
struct ReverseArgs {
    xs: Vec<isize>,
}

struct ReverseTest {
    endpoint: String,
}

impl Property for ReverseTest {
    type Args = ReverseArgs;
    type Return = bool;
    const PROPERTY_NAME: &'static str = "property_reverse";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prop1 = ReverseTest {
        endpoint: "http://[::1]:50051".to_string(),
    };

    let prop2 = ReverseTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    
    // quickcheck(prop).await;
    quickcheck_composite!(prop1, prop2, |args, results| { results[0] == results[1] });
    Ok(())
}
