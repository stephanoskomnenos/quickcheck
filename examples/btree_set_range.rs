use std::collections::BTreeSet;
use std::ops::Bound::{self, *};
use serde::{Deserialize, Serialize};
use quickcheck::{quickcheck, Arbitrary, Gen, Property};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BTreeSetRangeArgs {
    set: BTreeSet<i32>,
    range: (Bound<i32>, Bound<i32>),
}

impl Arbitrary for BTreeSetRangeArgs {
    fn arbitrary(g: &mut Gen) -> Self {
        let set: BTreeSet<i32> = Arbitrary::arbitrary(g);
        let range = (
            if bool::arbitrary(g) { Included(i32::arbitrary(g)) } else { Excluded(i32::arbitrary(g)) },
            if bool::arbitrary(g) { Included(i32::arbitrary(g)) } else { Excluded(i32::arbitrary(g)) },
        );
        BTreeSetRangeArgs { set, range }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new(
            self.set.shrink().zip(self.range.0.shrink().zip(self.range.1.shrink()))
                .map(|(new_set, (new_start, new_end))| {
                    BTreeSetRangeArgs { 
                        set: new_set, 
                        range: (new_start, new_end) 
                    }
                })
        )
    }
}

struct BTreeSetRangeTest {
    endpoint: String,
}

impl Property for BTreeSetRangeTest {
    type Args = BTreeSetRangeArgs;
    type Return = bool;
    const PROPERTY_NAME: &'static str = "property_btree_set_range";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prop = BTreeSetRangeTest {
        endpoint: "http://[::1]:50051".to_string(),
    };
    
    quickcheck(prop).await;
    Ok(())
}
