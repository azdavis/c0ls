use crate::ty::ty;
use crate::util::{comma_sep, must};
use event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) fn expr(p: &mut Parser<'_, SK>) {
  must(p, expr_opt)
}

pub(crate) fn expr_opt(p: &mut Parser<'_, SK>) -> Option<Exited> {
  expr_prec(p, 0)
}

const PRIM: [(SK, SK); 7] = [
  (SK::DecLit, SK::DecExpr),
  (SK::HexLit, SK::HexExpr),
  (SK::StringLit, SK::StringExpr),
  (SK::CharLit, SK::CharExpr),
  (SK::TrueKw, SK::TrueExpr),
  (SK::FalseKw, SK::FalseExpr),
  (SK::NullKw, SK::NullExpr),
];

fn expr_atom(p: &mut Parser<'_, SK>) -> Option<Exited> {
  for &(tok, node) in PRIM.iter() {
    if p.at(tok) {
      let entered = p.enter();
      p.bump();
      return Some(p.exit(entered, node));
    }
  }
  if p.at(SK::LRound) {
    let entered = p.enter();
    p.bump();
    expr(p);
    p.eat(SK::RRound);
    Some(p.exit(entered, SK::ParenExpr))
  } else if p.at(SK::Ident) {
    let entered = p.enter();
    p.bump();
    if p.at(SK::LRound) {
      p.bump();
      comma_sep(p, SK::Arg, expr);
      Some(p.exit(entered, SK::CallExpr))
    } else {
      Some(p.exit(entered, SK::IdentExpr))
    }
  } else if p.at(SK::AllocKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    ty(p);
    p.eat(SK::RRound);
    Some(p.exit(entered, SK::AllocExpr))
  } else if p.at(SK::AllocArrayKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    ty(p);
    p.eat(SK::Comma);
    expr(p);
    p.eat(SK::RRound);
    Some(p.exit(entered, SK::AllocArrayExpr))
  } else {
    None
  }
}

fn expr_prec(p: &mut Parser<'_, SK>, min_prec: u8) -> Option<Exited> {
  let mut exited =
    if p.at(SK::Bang) || p.at(SK::Tilde) || p.at(SK::Minus) || p.at(SK::Star) {
      assert!(!(UN_OP_PREC <= min_prec));
      let entered = p.enter();
      p.bump();
      must(p, |p| expr_prec(p, UN_OP_PREC - 1));
      p.exit(entered, SK::UnOpExpr)
    } else {
      expr_atom(p)?
    };
  loop {
    exited = if let Some(prec) = bin_op_prec(p) {
      if prec <= min_prec {
        break;
      }
      let entered = p.precede(exited);
      p.bump();
      must(p, |p| expr_prec(p, prec));
      p.exit(entered, SK::BinOpExpr)
    } else if p.at(SK::Question) {
      if min_prec != 0 {
        break;
      }
      let entered = p.precede(exited);
      p.bump();
      expr(p);
      p.eat(SK::Colon);
      expr(p);
      p.exit(entered, SK::TernaryExpr)
    } else if p.at(SK::Dot) {
      let entered = p.precede(exited);
      p.bump();
      p.eat(SK::Ident);
      p.exit(entered, SK::FieldGetExpr)
    } else if p.at(SK::Arrow) {
      let entered = p.precede(exited);
      p.bump();
      p.eat(SK::Ident);
      p.exit(entered, SK::DerefFieldGetExpr)
    } else if p.at(SK::LSquare) {
      let entered = p.precede(exited);
      p.bump();
      expr(p);
      p.eat(SK::RSquare);
      p.exit(entered, SK::SubscriptExpr)
    } else {
      break;
    };
  }
  Some(exited)
}

const UN_OP_PREC: u8 = 12;

fn bin_op_prec(p: &mut Parser<'_, SK>) -> Option<u8> {
  if p.at(SK::Star) || p.at(SK::Slash) || p.at(SK::Percent) {
    Some(11)
  } else if p.at(SK::Plus) || p.at(SK::Minus) {
    Some(10)
  } else if p.at(SK::LtLt) || p.at(SK::GtGt) {
    Some(9)
  } else if p.at(SK::Lt) || p.at(SK::LtEq) || p.at(SK::Gt) || p.at(SK::GtEq) {
    Some(8)
  } else if p.at(SK::EqEq) || p.at(SK::BangEq) {
    Some(7)
  } else if p.at(SK::And) {
    Some(6)
  } else if p.at(SK::Carat) {
    Some(5)
  } else if p.at(SK::Bar) {
    Some(4)
  } else if p.at(SK::AndAnd) {
    Some(3)
  } else if p.at(SK::BarBar) {
    Some(2)
  } else {
    None
  }
}

// no need for explicit TERNARY_PREC = 1 since it's the lowest and
// right-associative
