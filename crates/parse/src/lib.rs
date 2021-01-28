//! Parses tokens into a concrete syntax tree.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use syntax::event_parse::{Parser, Sink, Token};
use syntax::rowan::{GreenNodeBuilder, TextRange, TextSize};
use syntax::{SyntaxKind as SK, SyntaxNode};

mod expr;
mod item;
mod root;
mod stmt;
mod ty;
mod util;

#[derive(Debug)]
pub struct Parse {
  pub tree: SyntaxNode,
  pub errors: Vec<Error>,
}

#[derive(Debug)]
pub struct Error {
  pub range: TextRange,
  pub expected: Vec<SK>,
}

pub fn parse(tokens: Vec<Token<'_, SK>>) -> Parse {
  let mut p = Parser::new(tokens);
  root::root(&mut p);
  let mut sink = BuilderSink::default();
  p.finish(&mut sink);
  Parse {
    tree: SyntaxNode::new_root(sink.builder.finish()),
    errors: sink.errors,
  }
}

#[derive(Default)]
struct BuilderSink {
  builder: GreenNodeBuilder<'static>,
  range: Option<TextRange>,
  errors: Vec<Error>,
}

impl Sink<SK> for BuilderSink {
  fn enter(&mut self, kind: SK) {
    self.builder.start_node(kind.into());
  }

  fn token(&mut self, token: Token<'_, SK>) {
    self.builder.token(token.kind.into(), token.text);
    let start = self.range.as_ref().map_or(0.into(), |range| range.end());
    let end = start + TextSize::of(token.text);
    self.range = Some(TextRange::new(start, end));
  }

  fn exit(&mut self) {
    self.builder.finish_node();
  }

  fn error(&mut self, expected: Vec<SK>) {
    self.errors.push(Error {
      range: self.range.clone().expect("error with no tokens"),
      expected,
    });
  }
}
