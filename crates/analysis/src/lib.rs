//! Analysis of C0 source code.
//!
//! Doesn't do I/O, just takes in filenames and file contents and answers
//! queries about them.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

#[cfg(test)]
mod tests;

mod db;
mod types;
mod uses;

pub use db::Db;
pub use text_pos::{Position, Range};
pub use types::{CodeBlock, Diagnostic, Hover, Location};
