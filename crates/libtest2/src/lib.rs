//! An experimental replacement for libtest
//!
//! # Usage
//!
//! To use this, you most likely want to add a manual `[[test]]` section to
//! `Cargo.toml` and set `harness = false`. For example:
//!
//! ```toml
//! [[test]]
//! name = "mytest"
//! path = "tests/mytest.rs"
//! harness = false
//! ```
//!
//! And in `tests/mytest.rs` you would wrap `main` with [`#[main]`]:
//!
//! ```no_run
//! # use libtest2::RunError;
//! # use libtest2::RunResult;
//! # use libtest2::TestContext;
//! #[libtest2::test]
//! fn check_toph(_context: &TestContext) -> RunResult {
//!     Ok(())
//! }
//!
//! #[libtest2::main]
//! fn main() {
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
//#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

mod case;
mod macros;

#[doc(hidden)]
pub mod _private {
    pub use distributed_list::push;
    pub use distributed_list::DistributedList;
    pub use libtest2_harness::Case;
    pub use libtest2_harness::Source;
    pub use libtest2_harness::TestKind;

    pub use crate::_main_parse as main_parse;
    pub use crate::_test_parse as test_parse;
    pub use crate::case::DynCase;

    /// Static variable used to collect functions annotated with `#[libtest2::test]`
    ///
    /// Values are pushed to this by the [`crate::macros::test_parse`](`test_parse`) macro
    pub static TESTS: DistributedList<DynCase> = DistributedList::root();
}

pub use case::main;
pub use case::FnCase;
pub use libtest2_harness::RunError;
pub use libtest2_harness::RunResult;
pub use libtest2_harness::TestContext;
pub use libtest2_proc_macro::main;
pub use libtest2_proc_macro::test;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

/// Get an iterator to all collected test functions
///
/// Functions can be marked for collection by annotating them with the `#[libtest2::test]` macro
pub fn get_tests() -> impl Iterator<Item = case::DynCase> {
    _private::TESTS.iter().copied()
}
