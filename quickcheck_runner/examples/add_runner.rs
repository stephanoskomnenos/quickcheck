use serde::{Deserialize, Serialize};
use quickcheck_runner::quickcheck_runner_main;

#[derive(Deserialize, Serialize, Debug)]
struct AddArgs {
    a: i32,
    b: i32,
}

fn add_test(args: AddArgs) -> Result<i32, String> {
    let result = args.a + args.b;
    println!("{} + {} = {}", args.a, args.b, result);
    Ok(result)
}

// 使用宏创建runner
quickcheck_runner_main!(add_test, AddArgs, i32, "property_add");
