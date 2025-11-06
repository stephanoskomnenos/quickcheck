use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck, Arbitrary, Gen, RemoteTest};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OutOfBoundsArgs {
    length: usize,
    index: usize,
}

impl Arbitrary for OutOfBoundsArgs {
    fn arbitrary(g: &mut Gen) -> Self {
        OutOfBoundsArgs {
            length: usize::arbitrary(g) % 100, // Limit size for safety
            index: usize::arbitrary(g) % 100,
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.length.shrink().zip(self.index.shrink())
                .map(|(new_length, new_index)| OutOfBoundsArgs { 
                    length: new_length, 
                    index: new_index 
                })
        )
    }
}

struct OutOfBoundsTest {
    endpoint: String,
}

impl RemoteTest for OutOfBoundsTest {
    type Args = OutOfBoundsArgs;
    type Return = bool;
    const TEST_ID: &'static str = "out_of_bounds_test";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test = OutOfBoundsTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    
    quickcheck(test).await;
    Ok(())
}
