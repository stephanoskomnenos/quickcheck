use quickcheck_macros::Arbitrary;
use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck_composite, RemoteTest};

#[derive(Serialize, Deserialize, Debug, Clone, Arbitrary)]
struct ReverseArgs {
    xs: Vec<String>,
}

struct ReverseTest {
    endpoint: String,
}

impl RemoteTest for ReverseTest {
    type Args = ReverseArgs;
    type Return = Vec<String>;
    const TEST_ID: &'static str = "reverse_test";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() {
    let test1 = ReverseTest {
        endpoint: "http://[::1]:50051".to_string(),
    };

    // let test2 = ReverseTest {
    //     endpoint: "http://[::1]:50051".to_string(),
    // };
    
    // quickcheck_composite!(test1, test2, |args, results| { results[0] == results[1] });
    quickcheck_composite!(test1, |args, results| { results[0] == args.xs });
}
