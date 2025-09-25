/*!
This crate is a port of
[Haskell's QuickCheck](https://hackage.haskell.org/package/QuickCheck).

QuickCheck is a library for random testing of program properties. The
programmer provides a specification of the program, in the form of properties
which functions should satisfy, and QuickCheck then tests that the properties
hold in a large number of randomly generated cases.

For detailed examples, please see the
[README](https://github.com/BurntSushi/quickcheck).

# Compatibility

In general, this crate considers the `Arbitrary` implementations provided as
implementation details. Strategies may or may not change over time, which may
cause new test failures, presumably due to the discovery of new bugs due to a
new kind of witness being generated. These sorts of changes may happen in
semver compatible releases.
*/

// These re-exports remain the same.
pub use crate::arbitrary::{empty_shrinker, single_shrinker, Arbitrary, Gen};
pub use crate::tester::{quickcheck, QuickCheck, TestResult, Testable};

/// A macro for writing quickcheck tests.
///
/// This macro takes as input one or more property functions to test, and
/// produces a proper `#[test]` function for each property. The test functions
/// are now `async`.
///
/// # Example
///
/// ```rust
/// # #[macro_use] extern crate quickcheck;
/// # use tokio;
/// #
/// // The property function is now `async fn`.
/// async fn prop_reverse_reverse(xs: Vec<usize>) -> bool {
///     let rev: Vec<_> = xs.clone().into_iter().rev().collect();
///     let revrev: Vec<_> = rev.into_iter().rev().collect();
///     xs == revrev
/// }
///
/// # #[tokio::main]
/// # async fn main() {
/// #     // We need an async context to call the async quickcheck function.
/// #     // In a real test, `#[tokio::test]` would provide this.
/// #     quickcheck::quickcheck(prop_reverse_reverse as fn(Vec<usize>) -> _).await;
/// # }
/// ```
// #[macro_export]
// macro_rules! quickcheck {
//     // Internal rule, no changes needed.
//     (@as_items $($i:item)*) => ($($i)*);

//     // The macro's main matcher. It now accepts an optional `async` keyword.
//     {
//         $(
//             $(#[$m:meta])*
//             // The property function can now be `async fn`.
//             $(async)? fn $fn_name:ident($($arg_name:ident : $arg_ty:ty),*) -> $ret:ty {
//                 $($code:tt)*
//             }
//         )*
//     } => (
//         // The expansion logic.
//         $crate::quickcheck! {
//             @as_items
//             $(
//                 // The generated function is a standard test function.
//                 // Modern `#[test]` supports `async fn`.
//                 #[tokio::test]
//                 $(#[$m])*
//                 // The generated test function must be `async`.
//                 async fn $fn_name() {
//                     // The inner property function is also defined as `async`.
//                     async fn prop($($arg_name: $arg_ty),*) -> $ret {
//                         $($code)*
//                     }
//                     // The call to the main quickcheck runner is now awaited,
//                     // and the property is cast to an async function pointer.
//                     $crate::quickcheck(prop as fn($($arg_ty),*) -> _).await;
//                 }
//             )*
//         }
//     )
// }

// Logging features remain the same.
#[cfg(feature = "use_logging")]
fn env_logger_init() -> Result<(), log::SetLoggerError> {
    env_logger::try_init()
}
#[cfg(feature = "use_logging")]
macro_rules! info {
    ($($tt:tt)*) => {
        log::info!($($tt)*)
    };
}

#[cfg(not(feature = "use_logging"))]
fn env_logger_init() {}
#[cfg(not(feature = "use_logging"))]
macro_rules! info {
    ($($_ignore:tt)*) => {
        ()
    };
}

// Module declarations remain the same.
mod arbitrary;
mod composite;
mod tester;

#[cfg(test)]
mod tests;

// Re-export composite functionality
pub use composite::{CompositeProperty};
