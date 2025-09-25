use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck, Arbitrary, Gen, Property};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReverseArgs {
    xs: Vec<isize>,
}

impl Arbitrary for ReverseArgs {
    fn arbitrary(g: &mut Gen) -> Self {
        ReverseArgs {
            xs: Vec::<isize>::arbitrary(g),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(self.xs.shrink().map(|new_xs| ReverseArgs { xs: new_xs }))
    }
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
    let prop = ReverseTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    
    quickcheck(prop).await;
    Ok(())
}
