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
//!
//! # Known limitations and differences to the official test harness
//!
//! `libtest2` aims to be fully compatible with stable, non-deprecated parts of `libtest`
//! but there are differences for now.
//!
//! Some of the notable differences:
//!
//! - `#[test]` does not support all `Termination` types as return values,
//!   only what [`IntoRunResult`] supports.
//! - `#[ignore]` must come after the `#[test]` macro
//! - Output capture and `--no-capture`: simply not supported. The official
//!   `libtest` uses internal `std` functions to temporarily redirect output.
//!   `libtest` cannot use those, see also [libtest2#12](https://github.com/assert-rs/libtest2/issues/12)
//! - `--format=json` (unstable): our schema is part of an experiment to see what should be
//!   stabilized for `libtest`, see also [libtest2#42](https://github.com/assert-rs/libtest2/issues/42)

#![cfg_attr(docsrs, feature(doc_cfg))]
//#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

mod case;
mod macros;

#[doc(hidden)]
pub mod _private {
    pub use distributed_list::push;
    pub use distributed_list::DistributedList;
    pub use libtest2_harness::Source;
    pub use libtest2_harness::TestKind;

    pub use crate::_main_parse as main_parse;
    pub use crate::_parse_ignore as parse_ignore;
    pub use crate::_test_parse as test_parse;
    pub use crate::case::DynCase;
}

pub use case::main;
pub use case::FnCase;
pub use libtest2_harness::Case;
pub use libtest2_harness::IntoRunResult;
pub use libtest2_harness::RunError;
pub use libtest2_harness::RunResult;
pub use libtest2_harness::TestContext;
pub use libtest2_proc_macro::main;
pub use libtest2_proc_macro::test;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
