#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use std::ops::Range;
use syntax::event_parse::{Exited, Parser, Sink, Token};
use syntax::rowan::GreenNodeBuilder;
use syntax::{SyntaxKind as SK, SyntaxNode};

mod expr;
mod root;
mod stmt;
mod ty;

#[derive(Debug)]
pub struct Parse {
  pub tree: SyntaxNode,
  pub errors: Vec<Error>,
}

#[derive(Debug)]
pub struct Error {
  pub range: Range<usize>,
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
  range: Option<Range<usize>>,
  errors: Vec<Error>,
}

impl Sink<SK> for BuilderSink {
  fn enter(&mut self, kind: SK) {
    self.builder.start_node(kind.into());
  }

  fn token(&mut self, token: Token<'_, SK>) {
    self.builder.token(token.kind.into(), token.text.into());
    let start = self.range.as_ref().map_or(0, |range| range.end);
    self.range = Some(start..start + token.text.len());
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

type TypeDefs<'a> = std::collections::HashSet<&'a str>;

fn must<F>(p: &mut Parser<'_, SK>, f: F)
where
  F: FnOnce(&mut Parser<'_, SK>) -> Option<Exited>,
{
  if f(p).is_none() {
    p.error();
  }
}

fn comma_sep<F>(p: &mut Parser<'_, SK>, end: SK, mut f: F)
where
  F: FnMut(&mut Parser<'_, SK>),
{
  if p.at(end) {
    p.bump();
    return;
  }
  loop {
    f(p);
    if p.at(SK::Comma) {
      p.bump();
    } else if p.at(end) {
      p.bump();
      break;
    } else {
      p.error();
      break;
    }
  }
}
