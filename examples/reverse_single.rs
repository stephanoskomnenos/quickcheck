use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck, Arbitrary, Gen, RemoteTest};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReverseSingleArgs {
    xs: Vec<isize>,
}

impl Arbitrary for ReverseSingleArgs {
    fn arbitrary(g: &mut Gen) -> Self {
        ReverseSingleArgs {
            xs: Vec::<isize>::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.xs.shrink().map(|new_xs| ReverseSingleArgs { xs: new_xs }))
    }
}

struct ReverseSingleTest {
    endpoint: String,
}

impl RemoteTest for ReverseSingleTest {
    type Args = ReverseSingleArgs;
    type Return = bool;
    const TEST_ID: &'static str = "reverse_single_test";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test = ReverseSingleTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    
    quickcheck(test).await;
    Ok(())
}
