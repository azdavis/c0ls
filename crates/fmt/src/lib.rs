//! Formats a C0 CST.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

#[cfg(test)]
mod tests;

mod expr;
mod item;
mod root;
mod simp;
mod stmt;
mod ty;
mod util;

pub use root::get;
