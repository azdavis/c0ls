use crate::{ErrorKind, Parser};
use event_parse::Exited;
use syntax::SyntaxKind as SK;

pub(crate) fn must<F>(p: &mut Parser<'_>, f: F, ek: ErrorKind)
where
  F: FnOnce(&mut Parser<'_>) -> Option<Exited>,
{
  if f(p).is_none() {
    p.error(ek);
  }
}

pub(crate) fn comma_sep<F>(p: &mut Parser<'_>, wrap: SK, f: F)
where
  F: FnMut(&mut Parser<'_>),
{
  if p.at(SK::RRound) {
    p.bump();
    return;
  }
  many_sep(p, SK::Comma, wrap, f);
  p.eat(SK::RRound);
}

fn many_sep<F>(p: &mut Parser<'_>, sep: SK, wrap: SK, mut f: F)
where
  F: FnMut(&mut Parser<'_>),
{
  loop {
    let entered = p.enter();
    f(p);
    if p.at(sep) {
      p.bump();
      p.exit(entered, wrap);
    } else {
      p.exit(entered, wrap);
      break;
    }
  }
}
