use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck, Arbitrary, Gen, RemoteTest};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SortArgs {
    xs: Vec<isize>,
}

impl Arbitrary for SortArgs {
    fn arbitrary(g: &mut Gen) -> Self {
        SortArgs {
            xs: Vec::<isize>::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.xs.shrink().map(|new_xs| SortArgs { xs: new_xs }))
    }
}

struct SortTest {
    endpoint: String,
}

impl RemoteTest for SortTest {
    type Args = SortArgs;
    type Return = bool;
    const TEST_ID: &'static str = "sort_test";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test = SortTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    
    quickcheck(test).await;
    Ok(())
}
