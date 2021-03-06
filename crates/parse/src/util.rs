use syntax::event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) fn must<F>(p: &mut Parser<'_, SK>, f: F)
where
  F: FnOnce(&mut Parser<'_, SK>) -> Option<Exited>,
{
  if f(p).is_none() {
    p.error();
  }
}

pub(crate) fn comma_sep<F>(p: &mut Parser<'_, SK>, end: SK, wrap: SK, f: F)
where
  F: FnMut(&mut Parser<'_, SK>),
{
  if p.at(end) {
    p.bump();
    return;
  }
  many_sep(p, SK::Comma, wrap, f);
  p.eat(end);
}

fn many_sep<F>(p: &mut Parser<'_, SK>, sep: SK, wrap: SK, mut f: F)
where
  F: FnMut(&mut Parser<'_, SK>),
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
