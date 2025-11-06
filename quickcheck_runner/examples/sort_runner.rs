use serde::{Deserialize, Serialize};
use quickcheck_runner::quickcheck_runner_main;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct SortArgs {
    xs: Vec<isize>,
}

fn sort_test(args: SortArgs) -> Result<bool, String> {
    // 复制sort函数逻辑
    fn smaller_than<T: Clone + Ord>(xs: &[T], pivot: &T) -> Vec<T> {
        xs.iter().filter(|&x| *x < *pivot).cloned().collect()
    }

    fn larger_than<T: Clone + Ord>(xs: &[T], pivot: &T) -> Vec<T> {
        xs.iter().filter(|&x| *x > *pivot).cloned().collect()
    }

    fn sortk<T: Clone + Ord>(x: &T, xs: &[T]) -> Vec<T> {
        let mut result: Vec<T> = sort(&smaller_than(xs, x));
        let last_part = sort(&larger_than(xs, x));
        result.push(x.clone());
        result.extend(last_part.iter().cloned());
        result
    }

    fn sort<T: Clone + Ord>(list: &[T]) -> Vec<T> {
        if list.is_empty() {
            vec![]
        } else {
            sortk(&list[0], &list[1..])
        }
    }

    // 检查排序结果是否正确
    let sorted = sort(&args.xs);
    
    // 检查是否有序
    for win in sorted.windows(2) {
        if win[0] > win[1] {
            return Ok(false);
        }
    }
    
    // 检查长度是否一致
    if args.xs.len() != sorted.len() {
        return Ok(false);
    }
    
    Ok(true)
}

quickcheck_runner_main!(sort_test, SortArgs, bool, "sort_test");
