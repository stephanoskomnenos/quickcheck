use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck, Arbitrary, Gen, Property};

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

impl Property for SortTest {
    type Args = SortArgs;
    type Return = bool;
    const PROPERTY_NAME: &'static str = "property_sort";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prop = SortTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    
    quickcheck(prop).await;
    Ok(())
}
