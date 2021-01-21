use crate::util::{must, TypeDefs};
use syntax::event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) fn ty(p: &mut Parser<'_, SK>, tds: &TypeDefs<'_>) {
  must(p, |p| ty_opt(p, tds))
}

pub(crate) fn ty_opt(
  p: &mut Parser<'_, SK>,
  tds: &TypeDefs<'_>,
) -> Option<Exited> {
  ty_hd_opt(p, tds).map(|e| ty_tl(p, e))
}

pub(crate) fn ty_hd_opt(
  p: &mut Parser<'_, SK>,
  tds: &TypeDefs<'_>,
) -> Option<Exited> {
  if p.at(SK::IntKw)
    || p.at(SK::BoolKw)
    || p.at(SK::StringKw)
    || p.at(SK::CharKw)
    || p.at(SK::VoidKw)
  {
    let entered = p.enter();
    p.bump();
    Some(p.exit(entered, SK::PrimTy))
  } else if p.at(SK::StructKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::Ident);
    Some(p.exit(entered, SK::StructTy))
  } else if p.at(SK::Ident) && tds.contains(p.peek().unwrap().text) {
    // the one time we read from `tds`
    let entered = p.enter();
    p.bump();
    Some(p.exit(entered, SK::IdentTy))
  } else {
    None
  }
}

pub(crate) fn ty_tl(p: &mut Parser<'_, SK>, mut exited: Exited) -> Exited {
  loop {
    if p.at(SK::Star) {
      let entered = p.precede(exited);
      p.bump();
      exited = p.exit(entered, SK::PtrTy);
    } else if p.at(SK::LSquare) {
      let entered = p.precede(exited);
      p.bump();
      p.eat(SK::RSquare);
      exited = p.exit(entered, SK::ArrayTy);
    } else {
      break;
    }
  }
  exited
}
