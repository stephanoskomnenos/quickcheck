use serde::{Deserialize, Serialize};
use quickcheck_runner::quickcheck_runner_main;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct OutOfBoundsArgs {
    length: usize,
    index: usize,
}

fn out_of_bounds_test(args: OutOfBoundsArgs) -> Result<bool, String> {
    let v: Vec<_> = (0..args.length).collect();
    
    // 如果索引超出范围，应该panic
    if args.index >= args.length {
        // 这里会panic，被runner捕获
        let _ = v[args.index];
        Ok(true) // 这行代码不会执行
    } else {
        // 索引在范围内，测试应该被丢弃
        Ok(true)
    }
}

quickcheck_runner_main!(out_of_bounds_test, OutOfBoundsArgs, bool, "out_of_bounds_test");
