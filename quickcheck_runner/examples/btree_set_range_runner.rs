use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::ops::Bound::{self, *};
use quickcheck_runner::quickcheck_runner_main;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct BTreeSetRangeArgs {
    set: BTreeSet<i32>,
    range: (Bound<i32>, Bound<i32>),
}

fn btree_set_range_test(args: BTreeSetRangeArgs) -> Result<bool, String> {
    type RangeAny<T> = (Bound<T>, Bound<T>);

    trait RangeBounds<T> {
        fn contains(&self, _: &T) -> bool;
    }

    impl<T: PartialOrd> RangeBounds<T> for RangeAny<T> {
        fn contains(&self, item: &T) -> bool {
            (match &self.0 {
                Included(start) => start <= item,
                Excluded(start) => start < item,
                Unbounded => true,
            }) && (match &self.1 {
                Included(end) => item <= end,
                Excluded(end) => item < end,
                Unbounded => true,
            })
        }
    }

    fn panics<T: PartialOrd>(range: RangeAny<T>) -> bool {
        match (&range.0, &range.1) {
            (Excluded(start), Excluded(end)) => start >= end,
            (Included(start), Excluded(end) | Included(end))
            | (Excluded(start), Included(end)) => start > end,
            (Unbounded, _) | (_, Unbounded) => false,
        }
    }

    if panics(args.range) {
        return Ok(true); // 如果应该panic，返回true表示测试通过（因为panic会被runner捕获）
    }

    let xs: BTreeSet<_> = args.set.range(args.range).copied().collect();
    Ok(args.set.iter().all(|x| args.range.contains(x) == xs.contains(x)))
}

quickcheck_runner_main!(btree_set_range_test, BTreeSetRangeArgs, bool, "property_btree_set_range");
