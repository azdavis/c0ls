//! Parses tokens into a concrete syntax tree.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

mod expr;
mod item;
mod root;
mod simp;
mod stmt;
mod ty;
mod util;

use std::fmt;
use syntax::ast::{AstNode as _, Root};
use syntax::token::Token;
use syntax::SyntaxKind as SK;

/// The result of a parse.
#[derive(Debug)]
pub struct Parse {
  /// The root.
  pub root: Root,
  /// The errors encountered when parsing.
  pub errors: Vec<Error>,
}

/// A parse error.
pub type Error = event_parse::rowan_sink::Error<ErrorKind>;

/// A kind of error.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum ErrorKind {
  Kind(SK),
  Exp,
  Item,
  FnTail,
  Stmt,
  Ty,
}

impl event_parse::Expected<SK> for ErrorKind {
  fn expected(kind: SK) -> Self {
    ErrorKind::Kind(kind)
  }
}

pub(crate) type Parser<'a> = event_parse::Parser<'a, SK, ErrorKind>;

impl fmt::Display for ErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("expected ")?;
    match self {
      ErrorKind::Kind(kind) => kind.fmt(f),
      ErrorKind::Exp => f.write_str("an expression"),
      ErrorKind::Item => f.write_str("an item"),
      ErrorKind::FnTail => f.write_str("a function tail"),
      ErrorKind::Stmt => f.write_str("a statement"),
      ErrorKind::Ty => f.write_str("a type"),
    }
  }
}

/// Returns a parse of the tokens.
pub fn get(tokens: &[Token<'_, SK>]) -> Parse {
  let mut p = Parser::new(tokens);
  root::root(&mut p);
  let mut sink = event_parse::rowan_sink::RowanSink::default();
  p.finish(&mut sink);
  let (root, errors) = sink.finish();
  Parse {
    root: Root::cast(root).unwrap(),
    errors,
  }
}
