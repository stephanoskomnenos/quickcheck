use serde::{Deserialize, Serialize};
use quickcheck_runner::quickcheck_runner_main;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ReverseArgs {
    xs: Vec<isize>,
}

fn reverse_test(args: ReverseArgs) -> Result<bool, String> {
    let mut rev = vec![];
    for x in &args.xs {
        rev.insert(0, *x);
    }
    let revrev = {
        let mut revrev = vec![];
        for x in &rev {
            revrev.insert(0, *x);
        }
        // 故意设置的错误
        if revrev.len() % 3 == 1 {
            revrev[0] = 1;
        }
        revrev
    };
    Ok(revrev == args.xs)
}

quickcheck_runner_main!(reverse_test, ReverseArgs, bool, "property_reverse");
