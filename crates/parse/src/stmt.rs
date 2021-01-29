use crate::expr::{expr, expr_opt};
use crate::ty::ty_opt;
use crate::util::{must, TypeDefs};
use syntax::event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) fn stmt_block(p: &mut Parser<'_, SK>, tds: &TypeDefs) -> Exited {
  let entered = p.enter();
  p.eat(SK::LCurly);
  loop {
    if p.at(SK::RCurly) {
      p.bump();
      break;
    }
    if stmt_opt(p, tds).is_none() {
      p.error();
      break;
    }
  }
  p.exit(entered, SK::BlockStmt)
}

fn stmt(p: &mut Parser<'_, SK>, tds: &TypeDefs) {
  must(p, |p| stmt_opt(p, tds))
}

fn stmt_opt(p: &mut Parser<'_, SK>, tds: &TypeDefs) -> Option<Exited> {
  if p.at(SK::IfKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    expr(p, tds);
    p.eat(SK::RRound);
    stmt(p, tds);
    if p.at(SK::ElseKw) {
      let else_branch = p.enter();
      p.bump();
      stmt(p, tds);
      p.exit(else_branch, SK::ElseBranch);
    }
    Some(p.exit(entered, SK::IfStmt))
  } else if p.at(SK::WhileKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    expr(p, tds);
    p.eat(SK::RRound);
    stmt(p, tds);
    Some(p.exit(entered, SK::WhileStmt))
  } else if p.at(SK::ForKw) {
    // note we use SimpOpt to explicitly mark whether the simp was present or
    // not. we need to know which simp is the init and which is the step.
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    let simp_opt = p.enter();
    stmt_simple_opt(p, tds);
    p.exit(simp_opt, SK::SimpOpt);
    p.eat(SK::Semicolon);
    expr(p, tds);
    p.eat(SK::Semicolon);
    let simp_opt = p.enter();
    stmt_simple_opt(p, tds);
    p.exit(simp_opt, SK::SimpOpt);
    p.eat(SK::RRound);
    stmt(p, tds);
    Some(p.exit(entered, SK::ForStmt))
  } else if p.at(SK::ReturnKw) {
    let entered = p.enter();
    p.bump();
    expr_opt(p, tds);
    p.eat(SK::Semicolon);
    Some(p.exit(entered, SK::ReturnStmt))
  } else if p.at(SK::LCurly) {
    Some(stmt_block(p, tds))
  } else if p.at(SK::AssertKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    expr(p, tds);
    p.eat(SK::RRound);
    p.eat(SK::Semicolon);
    Some(p.exit(entered, SK::AssertStmt))
  } else if p.at(SK::ErrorKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    expr(p, tds);
    p.eat(SK::RRound);
    p.eat(SK::Semicolon);
    Some(p.exit(entered, SK::ErrorStmt))
  } else if p.at(SK::BreakKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::Semicolon);
    Some(p.exit(entered, SK::BreakStmt))
  } else if p.at(SK::ContinueKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::Semicolon);
    Some(p.exit(entered, SK::ContinueStmt))
  } else if let Some(exited) = stmt_simple_opt(p, tds) {
    let entered = p.precede(exited);
    p.eat(SK::Semicolon);
    Some(p.exit(entered, SK::SimpStmt))
  } else {
    None
  }
}

fn stmt_simple_opt(p: &mut Parser<'_, SK>, tds: &TypeDefs) -> Option<Exited> {
  if let Some(ty) = ty_opt(p, tds) {
    let entered = p.precede(ty);
    p.eat(SK::Ident);
    if p.at(SK::Eq) {
      let entered = p.enter();
      p.bump();
      expr(p, tds);
      p.exit(entered, SK::DefnTail);
    }
    return Some(p.exit(entered, SK::DeclSimp));
  }
  let exited = expr_opt(p, tds)?;
  let entered = p.precede(exited);
  // reject non-lv expr with assignment op semantically, not syntactically
  let completed = if p.at(SK::Eq)
    || p.at(SK::PlusEq)
    || p.at(SK::MinusEq)
    || p.at(SK::StarEq)
    || p.at(SK::SlashEq)
    || p.at(SK::PercentEq)
    || p.at(SK::LtLtEq)
    || p.at(SK::GtGtEq)
    || p.at(SK::AndEq)
    || p.at(SK::CaratEq)
    || p.at(SK::BarEq)
  {
    p.bump();
    expr(p, tds);
    p.exit(entered, SK::AsgnSimp)
  } else if p.at(SK::PlusPlus) || p.at(SK::MinusMinus) {
    p.bump();
    p.exit(entered, SK::IncDecSimp)
  } else {
    p.exit(entered, SK::ExprSimp)
  };
  Some(completed)
}
