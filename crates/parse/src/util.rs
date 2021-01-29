use std::collections::HashSet;
use syntax::event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

#[derive(Debug, Default)]
pub struct TypeDefs {
  store: HashSet<String>,
}

impl TypeDefs {
  pub(crate) fn insert(&mut self, s: String) {
    self.store.insert(s);
  }

  pub(crate) fn contains(&self, s: &str) -> bool {
    self.store.contains(s)
  }
}

pub(crate) fn must<F>(p: &mut Parser<'_, SK>, f: F)
where
  F: FnOnce(&mut Parser<'_, SK>) -> Option<Exited>,
{
  if f(p).is_none() {
    p.error();
  }
}

pub(crate) fn comma_sep<F>(p: &mut Parser<'_, SK>, end: SK, wrap: SK, mut f: F)
where
  F: FnMut(&mut Parser<'_, SK>),
{
  if p.at(end) {
    p.bump();
    return;
  }
  loop {
    let entered = p.enter();
    f(p);
    if p.at(SK::Comma) {
      p.bump();
      p.exit(entered, wrap);
    } else if p.at(end) {
      p.exit(entered, wrap);
      p.bump();
      break;
    } else {
      p.exit(entered, wrap);
      p.error();
      break;
    }
  }
}
