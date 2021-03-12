use crate::stmt::stmt_block;
use crate::ty::{ty, ty_hd_opt, ty_opt, ty_tl};
use crate::util::comma_sep;
use event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) fn item(p: &mut Parser<'_, SK>) {
  if p.at(SK::StructKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::Ident);
    if p.at(SK::Semicolon) {
      p.bump();
      p.exit(entered, SK::StructItem);
    } else if p.at(SK::LCurly) {
      let fields = p.enter();
      p.bump();
      loop {
        if p.at(SK::RCurly) {
          p.bump();
          break;
        }
        let ty = match ty_opt(p) {
          Some(x) => x,
          None => {
            p.error();
            break;
          }
        };
        let field = p.precede(ty);
        p.eat(SK::Ident);
        p.eat(SK::Semicolon);
        p.exit(field, SK::Field);
      }
      p.exit(fields, SK::Fields);
      p.eat(SK::Semicolon);
      p.exit(entered, SK::StructItem);
    } else {
      let ty_hd_exited = p.exit(entered, SK::StructTy);
      fn_tail(p, ty_hd_exited);
    }
  } else if p.at(SK::TypedefKw) {
    // see stmt_simple_opt for our approach to the typedef-name: identifier
    // problem.
    let entered = p.enter();
    p.bump();
    ty(p);
    p.eat(SK::Ident);
    p.eat(SK::Semicolon);
    p.exit(entered, SK::TypedefItem);
  } else if p.at(SK::Pragma) {
    let entered = p.enter();
    p.bump();
    p.exit(entered, SK::PragmaItem);
  } else if let Some(exited) = ty_hd_opt(p) {
    fn_tail(p, exited);
  } else {
    p.error()
  }
}

fn fn_tail(p: &mut Parser<'_, SK>, ty_hd_exited: Exited) {
  let ty_exited = ty_tl(p, ty_hd_exited);
  p.eat(SK::Ident);
  p.eat(SK::LRound);
  comma_sep(p, SK::Param, |p| {
    ty(p);
    p.eat(SK::Ident);
  });
  if p.at(SK::Semicolon) {
    let entered = p.precede(ty_exited);
    let semi = p.enter();
    p.bump();
    p.exit(semi, SK::SemicolonTail);
    p.exit(entered, SK::FnItem);
  } else if p.at(SK::LCurly) {
    let entered = p.precede(ty_exited);
    stmt_block(p);
    p.exit(entered, SK::FnItem);
  } else {
    p.error();
  }
}
