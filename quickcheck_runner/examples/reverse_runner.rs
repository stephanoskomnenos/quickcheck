use serde::{Deserialize, Serialize};
use quickcheck_runner::quickcheck_runner_main;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ReverseArgs {
    xs: Vec<String>,
}

fn reverse_test(args: ReverseArgs) -> Result<Vec<String>, String> {
    let mut rev = vec![];
    for x in &args.xs {
        rev.insert(0, x);
    }
    let revrev = {
        let mut revrev = vec![];
        for x in rev {
            revrev.insert(0, x.to_owned());
        }
        // 故意设置的错误
        if revrev.len() % 2 == 1 {
            revrev[0] = "Here is an error".to_owned();
        }
        revrev
    };
    Ok(revrev)
}

quickcheck_runner_main!(reverse_test, ReverseArgs, Vec<String>, "property_reverse");
