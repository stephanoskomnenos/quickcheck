// use std::collections::hash_map::DefaultHasher;
// use std::collections::{HashMap, HashSet};
// use std::ffi::CString;
// use std::hash::BuildHasherDefault;
// use std::path::PathBuf;

// use super::{quickcheck, Gen, QuickCheck, TestResult};

// #[tokio::test]
// async fn prop_oob() {
//     async fn prop() -> bool {
//         let zero: Vec<bool> = vec![];
//         zero[0] // This will panic
//     }
//     // quicktest is now async and must be awaited.
//     if let Ok(n) = QuickCheck::new().quicktest(prop as fn() -> _).await {
//         panic!(
//             "prop_oob should fail with a runtime error but instead it passed {} tests.",
//             n
//         );
//     }
// }

// #[tokio::test]
// async fn prop_reverse_reverse() {
//     async fn prop(xs: Vec<usize>) -> bool {
//         let rev: Vec<_> = xs.clone().into_iter().rev().collect();
//         let revrev: Vec<_> = rev.into_iter().rev().collect();
//         xs == revrev
//     }
//     quickcheck(prop as fn(Vec<usize>) -> _).await;
// }

// // The quickcheck! macro now generates async tests.
// // The properties themselves must be declared as `async fn`.
// quickcheck! {
//     async fn prop_reverse_reverse_macro(xs: Vec<usize>) -> bool {
//         let rev: Vec<_> = xs.clone().into_iter().rev().collect();
//         let revrev: Vec<_> = rev.into_iter().rev().collect();
//         xs == revrev
//     }

//     #[should_panic]
//     async fn prop_macro_panic(_x: u32) -> bool {
//         assert!(false);
//         false
//     }
// }

// #[tokio::test]
// async fn reverse_single() {
//     async fn prop(xs: Vec<usize>) -> TestResult {
//         if xs.len() != 1 {
//             TestResult::discard()
//         } else {
//             TestResult::from_bool(
//                 xs == xs.clone().into_iter().rev().collect::<Vec<_>>(),
//             )
//         }
//     }
//     quickcheck(prop as fn(Vec<usize>) -> _).await;
// }

// #[tokio::test]
// async fn reverse_app() {
//     async fn prop(xs: Vec<usize>, ys: Vec<usize>) -> bool {
//         let mut app = xs.clone();
//         app.extend(ys.iter().copied());
//         let app_rev: Vec<usize> = app.into_iter().rev().collect();

//         let rxs: Vec<usize> = xs.into_iter().rev().collect();
//         let mut rev_app = ys.into_iter().rev().collect::<Vec<usize>>();
//         rev_app.extend(rxs);

//         app_rev == rev_app
//     }
//     quickcheck(prop as fn(Vec<usize>, Vec<usize>) -> _).await;
// }

// #[tokio::test]
// async fn max() {
//     async fn prop(x: isize, y: isize) -> TestResult {
//         if x > y {
//             TestResult::discard()
//         } else {
//             TestResult::from_bool(::std::cmp::max(x, y) == y)
//         }
//     }
//     quickcheck(prop as fn(isize, isize) -> _).await;
// }

// #[tokio::test]
// async fn sort() {
//     async fn prop(mut xs: Vec<isize>) -> bool {
//         xs.sort_unstable();
//         for i in xs.windows(2) {
//             if i[0] > i[1] {
//                 return false;
//             }
//         }
//         true
//     }
//     quickcheck(prop as fn(Vec<isize>) -> _).await;
// }

// // Helper functions `sieve` and `is_prime` do not need to be async.
// fn sieve(n: usize) -> Vec<usize> {
//     if n <= 1 { return vec![]; }
//     let mut marked = vec![false; n + 1];
//     marked[0] = true;
//     marked[1] = true;
//     marked[2] = true;
//     for p in 2..n {
//         for i in (2 * p..n).filter(|&n| n % p == 0) {
//             marked[i] = true;
//         }
//     }
//     marked.iter().enumerate().filter_map(|(i, &m)| if m { None } else { Some(i) }).collect()
// }

// fn is_prime(n: usize) -> bool {
//     n != 0 && n != 1 && (2..).take_while(|i| i * i <= n).all(|i| n % i != 0)
// }

// #[tokio::test]
// #[should_panic]
// async fn sieve_not_prime() {
//     async fn prop_all_prime(n: u8) -> bool {
//         sieve(n as usize).into_iter().all(is_prime)
//     }
//     quickcheck(prop_all_prime as fn(u8) -> _).await;
// }

// #[tokio::test]
// #[should_panic]
// async fn sieve_not_all_primes() {
//     async fn prop_prime_iff_in_the_sieve(n: u8) -> bool {
//         let n = n as usize;
//         sieve(n) == (0..=n).filter(|&i| is_prime(i)).collect::<Vec<_>>()
//     }
//     quickcheck(prop_prime_iff_in_the_sieve as fn(u8) -> _).await;
// }

// #[tokio::test]
// async fn testable_result() {
//     async fn result() -> Result<bool, String> {
//         Ok(true)
//     }
//     quickcheck(result as fn() -> _).await;
// }

// #[tokio::test]
// #[should_panic]
// async fn testable_result_err() {
//     async fn prop(_: i32) -> Result<bool, i32> {
//         Err(42) // A property that always returns an error
//     }
//     quickcheck(prop as fn(i32) -> _).await;
// }

// #[tokio::test]
// async fn testable_unit() {
//     async fn do_nothing() {}
//     quickcheck(do_nothing as fn() -> _).await;
// }

// #[tokio::test]
// async fn testable_unit_panic() {
//     async fn panic() {
//         panic!();
//     }
//     assert!(QuickCheck::new().quicktest(panic as fn() -> _).await.is_err());
// }

// #[tokio::test]
// async fn regression_issue_83() {
//     async fn prop(_: u8) -> bool { true }
//     QuickCheck::new().set_rng(Gen::new(1024)).quickcheck(prop as fn(u8) -> _).await;
// }

// #[tokio::test]
// async fn regression_issue_83_signed() {
//     async fn prop(_: i8) -> bool { true }
//     QuickCheck::new().set_rng(Gen::new(1024)).quickcheck(prop as fn(i8) -> _).await;
// }

// #[tokio::test]
// #[should_panic(expected = "foo")]
// async fn panic_msg_1() {
//     async fn prop() -> bool { panic!("foo"); }
//     quickcheck(prop as fn() -> _).await;
// }

// #[tokio::test]
// #[should_panic(expected = "foo")]
// async fn panic_msg_2() {
//     async fn prop() -> bool {
//         assert!("foo" == "bar");
//         true
//     }
//     quickcheck(prop as fn() -> _).await;
// }

// #[tokio::test]
// #[should_panic(expected = "foo")]
// async fn panic_msg_3() {
//     async fn prop() -> bool {
//         assert_eq!("foo", "bar");
//         true
//     }
//     quickcheck(prop as fn() -> _).await;
// }

// #[tokio::test]
// #[should_panic]
// async fn regression_issue_107_hang() {
//     async fn prop(a: Vec<u8>) -> bool { a.contains(&1) }
//     quickcheck(prop as fn(_) -> _).await;
// }

// #[tokio::test]
// #[should_panic(expected = "(Unable to generate enough tests, 0 not discarded.)")]
// async fn all_tests_discarded_min_tests_passed_set() {
//     async fn prop_discarded(_: u8) -> TestResult { TestResult::discard() }
//     QuickCheck::new()
//         .tests(16)
//         .min_tests_passed(8)
//         .quickcheck(prop_discarded as fn(u8) -> _)
//         .await;
// }

// #[tokio::test]
// async fn all_tests_discarded_min_tests_passed_missing() {
//     async fn prop_discarded(_: u8) -> TestResult { TestResult::discard() }
//     QuickCheck::new().quickcheck(prop_discarded as fn(u8) -> _).await;
// }

// // Properties inside the macro must also be `async`.
// quickcheck! {
//     async fn pathbuf(_p: PathBuf) -> bool {
//         true
//     }

//     async fn basic_hashset(_set: HashSet<u8>) -> bool {
//         true
//     }

//     async fn basic_hashmap(_map: HashMap<u8, u8>) -> bool {
//         true
//     }

//     async fn substitute_hashset(
//         _set: HashSet<u8, BuildHasherDefault<DefaultHasher>>
//     ) -> bool {
//         true
//     }

//     async fn substitute_hashmap(
//         _map: HashMap<u8, u8, BuildHasherDefault<DefaultHasher>>
//     ) -> bool {
//         true
//     }

//     async fn cstring(_p: CString) -> bool {
//         true
//     }
// }
use crate::{quickcheck, quickcheck_composite, tester::RemoteTest, Arbitrary, Gen};
use serde::{Serialize, Deserialize};

const ENDPOINT: &str = "http://[::1]:50051";

// 1. Define a struct for your test's arguments.
//    It must derive Arbitrary, Serialize, and other traits.

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReverseArgs {
    xs: Vec<usize>,
}

// You would implement Arbitrary like this:
impl Arbitrary for ReverseArgs {
    fn arbitrary(g: &mut Gen) -> Self {
        // To create a random ReverseArgs, we need a random `Vec<usize>`.
        // We can get one by calling Vec::<usize>::arbitrary(g).
        ReverseArgs {
            xs: Vec::<usize>::arbitrary(g),
        }
    }

    // `shrink` method goes here...
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        // (See next section)
        Box::new(self.xs.shrink().map(|new_xs| ReverseArgs { xs: new_xs }))
    }
}

// 2. Define a struct to represent your test. It will hold the runner's address.
struct ReverseTest {
    endpoint: String,
}

// 3. Implement the `RemoteTest` trait to link everything together.
impl RemoteTest for ReverseTest {
    // Link to the argument struct.
    type Args = ReverseArgs;
    type Return = Vec<usize>;
    
    // Set the unique name for the runner to identify this test.
    const TEST_ID: &'static str = "reverse_list_test";

    fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

// Now you can run the test!
#[tokio::test]
#[ignore] // Run this test manually when the gRPC runner is active.
async fn test_the_reverse_test() {
    let test = ReverseTest {
        endpoint: ENDPOINT.to_string(),
    };

    // `quickcheck` can accept `test` directly because it implements `Testable` via `RemoteTest`.
    quickcheck(test).await;
}

// --- Example with multiple arguments ---

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AddArgs {
    a: i64,
    b: i64,
}

impl Arbitrary for AddArgs {
    fn arbitrary(g: &mut Gen) -> Self {
        Self {
            a: i64::arbitrary(g),
            b: i64::arbitrary(g),
        }
    }
    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let b = self.b;
        Box::new(
            self.a.shrink().map(move |new_a| AddArgs { a: new_a, b }),
        )
    }
}

struct AddTest {
    endpoint: String,
}

impl RemoteTest for AddTest {
    type Args = AddArgs;
    type Return = i64;
    const TEST_ID: &'static str = "add_test";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::test]
#[ignore]
async fn test_the_add_test() {
    let test1 = AddTest { endpoint: ENDPOINT.to_string() };
    let test2 = AddTest { endpoint: ENDPOINT.to_string() };
    // quickcheck(test1).await;
    quickcheck_composite!(test1, test2, |args, results| { 
        let equal = results[0] == results[1] && results[0] == args.a + args.b;
        if !equal {
            println!("Results: {:?}", results);
        }
        equal
    });
}

// --- Test panic handling ---
struct PanicTest {
    endpoint: String,
}

impl RemoteTest for PanicTest {
    type Args = AddArgs;  // Reuse AddArgs for simplicity
    type Return = i32;
    const TEST_ID: &'static str = "panic_test";
    fn endpoint(&self) -> &str { &self.endpoint }
}

#[tokio::test]
#[ignore] // Run this test manually when the gRPC runner is active.
async fn test_panic_handling() {
    let test1 = PanicTest { endpoint: ENDPOINT.to_string() };
    let test2 = PanicTest { endpoint: ENDPOINT.to_string() };
    // This should not panic at the test level - the panic should be caught by the runner
    // and treated as a test failure, which will then go through the shrink process
    // quickcheck(test).await;
    quickcheck_composite!(test1, test2, |_args, _results| { false });
}
