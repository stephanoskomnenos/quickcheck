use serde::{Deserialize, Serialize};
use quickcheck_runner::quickcheck_runner_main;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct ReverseSingleArgs {
    xs: Vec<isize>,
}

fn reverse_single_test(args: ReverseSingleArgs) -> Result<bool, String> {
    // 如果向量长度不为1，则丢弃测试
    if args.xs.len() != 1 {
        return Ok(true); // 在runner中，丢弃测试通过返回true
    }
    
    // 反转函数
    fn reverse<T: Clone>(xs: &[T]) -> Vec<T> {
        let mut rev = vec![];
        for x in xs {
            rev.insert(0, x.clone());
        }
        rev
    }
    
    // 检查反转后是否等于自身
    Ok(args.xs == reverse(&args.xs))
}

quickcheck_runner_main!(reverse_single_test, ReverseSingleArgs, bool, "property_reverse_single");
