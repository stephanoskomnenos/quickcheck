use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck, Arbitrary, Gen, RemoteTest};

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

impl RemoteTest for SieveTest {
    type Args = SieveArgs;
    type Return = bool;
    const TEST_ID: &'static str = "sieve_test";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test = SieveTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    
    quickcheck(test).await;
    Ok(())
}
