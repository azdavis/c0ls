use crate::stmt::stmt_block;
use crate::ty::{ty, ty_hd_opt, ty_opt, ty_tl};
use crate::util::{comma_sep, must, TypeDefs};
use syntax::event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) fn root(p: &mut Parser<'_, SK>) {
  let entered = p.enter();
  let mut tds = TypeDefs::new();
  while p.peek().is_some() {
    item(p, &mut tds);
  }
  p.exit(entered, SK::Root);
}

fn item<'input>(p: &mut Parser<'input, SK>, tds: &mut TypeDefs<'input>) {
  if p.at(SK::StructKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::Ident);
    if p.at(SK::Semicolon) {
      p.bump();
      p.exit(entered, SK::StructDeclItem);
    } else if p.at(SK::LCurly) {
      p.bump();
      loop {
        if p.at(SK::RCurly) {
          p.bump();
          break;
        }
        if param_opt(p, tds).is_none() {
          p.error();
          break;
        }
        p.eat(SK::Semicolon);
      }
      p.eat(SK::Semicolon);
      p.exit(entered, SK::StructDefnItem);
    } else {
      let ty_hd_exited = p.exit(entered, SK::StructTy);
      fn_tail(p, tds, ty_hd_exited);
    }
  } else if p.at(SK::TypedefKw) {
    let entered = p.enter();
    p.bump();
    ty(p, tds);
    if let Some(tok) = p.eat(SK::Ident) {
      // the one time we mutate `tds`
      tds.insert(tok.text);
    }
    p.eat(SK::Semicolon);
    p.exit(entered, SK::TypeDefnItem);
  } else if p.at(SK::UseKw) {
    let entered = p.enter();
    p.bump();
    if p.at(SK::LibLit) || p.at(SK::StringLit) {
      p.bump();
    } else {
      p.error();
    }
    p.exit(entered, SK::UseItem);
  } else if let Some(exited) = ty_hd_opt(p, tds) {
    fn_tail(p, tds, exited);
  } else {
    p.error()
  }
}

fn fn_tail(p: &mut Parser<'_, SK>, tds: &TypeDefs<'_>, ty_hd_exited: Exited) {
  let ty_exited = ty_tl(p, ty_hd_exited);
  p.eat(SK::Ident);
  p.eat(SK::LRound);
  comma_sep(p, SK::RRound, |p| param(p, tds));
  if p.at(SK::Semicolon) {
    let entered = p.precede(ty_exited);
    p.bump();
    p.exit(entered, SK::FnDeclItem);
  } else if p.at(SK::LCurly) {
    let entered = p.precede(ty_exited);
    stmt_block(p, tds);
    p.exit(entered, SK::FnDefnItem);
  } else {
    p.error();
  }
}

fn param(p: &mut Parser<'_, SK>, tds: &TypeDefs<'_>) {
  must(p, |p| param_opt(p, tds))
}

fn param_opt(p: &mut Parser<'_, SK>, tds: &TypeDefs<'_>) -> Option<Exited> {
  let exited = ty_opt(p, tds)?;
  let entered = p.precede(exited);
  p.eat(SK::Ident);
  Some(p.exit(entered, SK::Param))
}
