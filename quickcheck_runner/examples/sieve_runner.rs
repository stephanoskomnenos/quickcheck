use serde::{Deserialize, Serialize};
use quickcheck_runner::quickcheck_runner_main;

#[derive(Deserialize, Serialize, Debug, Clone)]
struct SieveArgs {
    n: usize,
}

fn sieve_test(args: SieveArgs) -> Result<bool, String> {
    fn sieve(n: usize) -> Vec<usize> {
        if n <= 1 {
            return vec![];
        }

        let mut marked = vec![false; n + 1];
        marked[0] = true;
        marked[1] = true;
        marked[2] = true;
        for p in 2..n {
            for i in (2 * p..n).filter(|&n| n % p == 0) {
                marked[i] = true;
            }
        }
        marked
            .iter()
            .enumerate()
            .filter_map(|(i, &m)| if m { None } else { Some(i) })
            .collect()
    }

    fn is_prime(n: usize) -> bool {
        n != 0 && n != 1 && (2..).take_while(|i| i * i <= n).all(|i| n % i != 0)
    }

    // 检查筛法结果是否都是素数
    let primes = sieve(args.n);
    Ok(primes.into_iter().all(is_prime))
}

quickcheck_runner_main!(sieve_test, SieveArgs, bool, "sieve_test");
