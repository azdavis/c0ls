use syntax::event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) type TypeDefs<'a> = std::collections::HashSet<&'a str>;

pub(crate) fn must<F>(p: &mut Parser<'_, SK>, f: F)
where
  F: FnOnce(&mut Parser<'_, SK>) -> Option<Exited>,
{
  if f(p).is_none() {
    p.error();
  }
}

pub(crate) fn comma_sep<F>(p: &mut Parser<'_, SK>, end: SK, mut f: F)
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
