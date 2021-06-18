//! Types for working with C0 syntax trees.

#![deny(rust_2018_idioms)]

pub mod ast;
mod kind;

pub use kind::*;
pub use rowan;
pub use token;

#[derive(Debug, Clone)]
pub struct Use {
  pub kind: UseKind,
  pub range: rowan::TextRange,
  /// would be nice for this to be borrowed from the input instead of an owned
  /// String. but, we don't process the uses until we have all the files
  /// together. it's inconvenient for the borrow to live that long.
  pub path: String,
}

#[derive(Debug, Clone, Copy)]
pub enum UseKind {
  Local,
  Lib,
}
