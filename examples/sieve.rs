use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck, Arbitrary, Gen, Property};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SieveArgs {
    n: usize,
}

impl Arbitrary for SieveArgs {
    fn arbitrary(g: &mut Gen) -> Self {
        SieveArgs {
            n: usize::arbitrary(g) % 100, // Limit size for performance
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.n.shrink().map(|new_n| SieveArgs { n: new_n }))
    }
}

struct SieveTest {
    endpoint: String,
}

impl Property for SieveTest {
    type Args = SieveArgs;
    type Return = bool;
    const PROPERTY_NAME: &'static str = "property_sieve";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prop = SieveTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    
    quickcheck(prop).await;
    Ok(())
}
