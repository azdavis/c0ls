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

use event_parse::Parser;
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
pub type Error = event_parse::rowan_sink::Error<SK>;

/// A list of expected tokens.
#[derive(Debug)]
pub struct Expected {
  /// The token kinds.
  pub kinds: Vec<SK>,
}

impl fmt::Display for Expected {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut iter = self.kinds.iter();
    write!(f, "expected any of ")?;
    if let Some(kind) = iter.next() {
      write!(f, "{}", kind.token_desc().unwrap_or("<non-token>"))?;
    }
    for kind in iter {
      write!(f, ", {}", kind.token_desc().unwrap_or("<non-token>"))?;
    }
    Ok(())
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
