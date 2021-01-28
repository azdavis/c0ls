//! Performs static analysis on an abstract syntax tree.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

mod ck;
pub mod error;
pub mod name;
pub mod ty;
pub mod util;

pub use ck::root::get;
