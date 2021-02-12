//! Types for working with C0 syntax trees.

#![deny(rust_2018_idioms)]

pub mod ast;
mod kind;

pub use event_parse;
pub use kind::*;
pub use rowan;

#[derive(Debug)]
pub struct Use<'a> {
  pub kind: UseKind,
  pub range: rowan::TextRange,
  pub path: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub enum UseKind {
  Local,
  Lib,
}
