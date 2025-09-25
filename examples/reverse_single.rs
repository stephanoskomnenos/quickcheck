use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck, Arbitrary, Gen, Property};

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

impl Property for ReverseSingleTest {
    type Args = ReverseSingleArgs;
    type Return = bool;
    const PROPERTY_NAME: &'static str = "property_reverse_single";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prop = ReverseSingleTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    
    quickcheck(prop).await;
    Ok(())
}
