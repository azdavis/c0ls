//! Analysis of C0 source code.
//!
//! Doesn't do I/O, just takes in filenames and file contents and answers
//! queries about them.

#![deny(rust_2018_idioms)]

mod db;
mod lines;
mod types;
mod uri;
mod uses;

pub use db::Db;
pub use types::{Diagnostic, Hover, Location, Markdown, Position, Range};
pub use uri::Uri;