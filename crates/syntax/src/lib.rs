//! Types for working with C0 syntax trees.

#![deny(rust_2018_idioms)]

mod generated;
pub use generated::*;

#[derive(Debug)]
pub struct Use {
  pub kind: UseKind,
  pub range: rowan::TextRange,
  pub path: String,
}

#[derive(Debug, Clone, Copy)]
pub enum UseKind {
  Local,
  Lib,
}
