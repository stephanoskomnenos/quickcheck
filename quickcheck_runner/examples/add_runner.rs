use serde::{Deserialize, Serialize};
use quickcheck_runner::quickcheck_runner_main;

#[derive(Deserialize, Serialize, Debug)]
struct AddArgs {
    a: i64,
    b: i64,
}

fn add_test(args: AddArgs) -> Result<i64, String> {
    let result = args.a + args.b;
    println!("{} + {} = {}", args.a, args.b, result);
    Ok(result)
}

// 使用宏创建runner
quickcheck_runner_main!(add_test, AddArgs, i64, "add_test");
