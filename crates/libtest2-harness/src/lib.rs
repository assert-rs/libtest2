//! An experimental replacement for the core of libtest

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::print_stderr)]
// #![warn(clippy::print_stdout)]

mod case;
mod context;
mod error;
mod harness;
mod notify;

pub mod cli;

pub use case::*;
pub use context::*;
pub use error::*;
pub use harness::*;
pub use notify::RunMode;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
