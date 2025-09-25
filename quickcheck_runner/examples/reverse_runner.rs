use serde::{Deserialize, Serialize};
use quickcheck_runner::quickcheck_runner_main;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ReverseArgs {
    xs: Vec<isize>,
}

fn reverse_test(args: ReverseArgs) -> Result<Vec<isize>, String> {
    let mut rev = vec![];
    for x in &args.xs {
        rev.insert(0, *x);
    }
    let revrev = {
        let mut revrev = vec![];
        for x in &rev {
            revrev.insert(0, *x);
        }
        revrev
    };
    Ok(revrev)
}

quickcheck_runner_main!(reverse_test, ReverseArgs, Vec<isize>, "property_reverse");
