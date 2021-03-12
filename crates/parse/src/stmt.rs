use crate::expr::{expr, expr_opt};
use crate::simp::simp_opt;
use crate::util::must;
use event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) fn stmt_block(p: &mut Parser<'_, SK>) -> Exited {
  let entered = p.enter();
  p.eat(SK::LCurly);
  loop {
    if p.at(SK::RCurly) {
      p.bump();
      break;
    }
    if stmt_opt(p).is_none() {
      p.error();
      break;
    }
  }
  p.exit(entered, SK::BlockStmt)
}

fn stmt(p: &mut Parser<'_, SK>) {
  must(p, stmt_opt)
}

fn stmt_opt(p: &mut Parser<'_, SK>) -> Option<Exited> {
  if p.at(SK::IfKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    expr(p);
    p.eat(SK::RRound);
    stmt(p);
    if p.at(SK::ElseKw) {
      let else_branch = p.enter();
      p.bump();
      stmt(p);
      p.exit(else_branch, SK::ElseBranch);
    }
    Some(p.exit(entered, SK::IfStmt))
  } else if p.at(SK::WhileKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    expr(p);
    p.eat(SK::RRound);
    stmt(p);
    Some(p.exit(entered, SK::WhileStmt))
  } else if p.at(SK::ForKw) {
    // note we use SimpOpt to explicitly mark whether the simp was present or
    // not. we need to know which simp is the init and which is the step.
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    let simp_opt_entered = p.enter();
    simp_opt(p);
    p.exit(simp_opt_entered, SK::SimpOpt);
    p.eat(SK::Semicolon);
    expr(p);
    p.eat(SK::Semicolon);
    let simp_opt_entered = p.enter();
    simp_opt(p);
    p.exit(simp_opt_entered, SK::SimpOpt);
    p.eat(SK::RRound);
    stmt(p);
    Some(p.exit(entered, SK::ForStmt))
  } else if p.at(SK::ReturnKw) {
    let entered = p.enter();
    p.bump();
    expr_opt(p);
    p.eat(SK::Semicolon);
    Some(p.exit(entered, SK::ReturnStmt))
  } else if p.at(SK::LCurly) {
    Some(stmt_block(p))
  } else if p.at(SK::AssertKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    expr(p);
    p.eat(SK::RRound);
    p.eat(SK::Semicolon);
    Some(p.exit(entered, SK::AssertStmt))
  } else if p.at(SK::ErrorKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    expr(p);
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
  } else if let Some(exited) = simp_opt(p) {
    let entered = p.precede(exited);
    p.eat(SK::Semicolon);
    Some(p.exit(entered, SK::SimpStmt))
  } else {
    None
  }
}
