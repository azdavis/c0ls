use crate::ty::ty;
use crate::util::{comma_sep, must, TypeDefs};
use syntax::event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) fn expr(p: &mut Parser<'_, SK>, tds: &TypeDefs) {
  must(p, |p| expr_opt(p, tds))
}

pub(crate) fn expr_opt(
  p: &mut Parser<'_, SK>,
  tds: &TypeDefs,
) -> Option<Exited> {
  expr_prec(p, tds, 0)
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

fn expr_hd(p: &mut Parser<'_, SK>, tds: &TypeDefs) -> Option<Exited> {
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
    expr(p, tds);
    p.eat(SK::RRound);
    Some(p.exit(entered, SK::ParenExpr))
  } else if p.at(SK::Ident) {
    let entered = p.enter();
    p.bump();
    if p.at(SK::LRound) {
      p.bump();
      comma_sep(p, SK::RRound, SK::Arg, |p| expr(p, tds));
      Some(p.exit(entered, SK::CallExpr))
    } else {
      Some(p.exit(entered, SK::IdentExpr))
    }
  } else if p.at(SK::Bang)
    || p.at(SK::Tilde)
    || p.at(SK::Minus)
    || p.at(SK::Star)
  {
    let entered = p.enter();
    p.bump();
    // higher than any infix op prec
    must(p, |p| expr_prec(p, tds, 11));
    Some(p.exit(entered, SK::UnOpExpr))
  } else if p.at(SK::AllocKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    ty(p, tds);
    p.eat(SK::RRound);
    Some(p.exit(entered, SK::AllocExpr))
  } else if p.at(SK::AllocArrayKw) {
    let entered = p.enter();
    p.bump();
    p.eat(SK::LRound);
    ty(p, tds);
    p.eat(SK::Comma);
    expr(p, tds);
    p.eat(SK::RRound);
    Some(p.exit(entered, SK::AllocArrayExpr))
  } else {
    None
  }
}

fn expr_prec(
  p: &mut Parser<'_, SK>,
  tds: &TypeDefs,
  min_prec: u8,
) -> Option<Exited> {
  let mut exited = expr_hd(p, tds)?;
  loop {
    exited = if let Some(prec) = infix_prec(p) {
      if prec <= min_prec {
        break;
      }
      let entered = p.precede(exited);
      p.bump();
      must(p, |p| expr_prec(p, tds, prec));
      p.exit(entered, SK::BinOpExpr)
    } else if p.at(SK::Question) {
      if min_prec != 0 {
        break;
      }
      let entered = p.precede(exited);
      p.bump();
      expr(p, tds);
      p.eat(SK::Colon);
      expr(p, tds);
      p.exit(entered, SK::TernaryExpr)
    } else if p.at(SK::Dot) {
      let entered = p.precede(exited);
      p.bump();
      p.eat(SK::Ident);
      p.exit(entered, SK::DotExpr)
    } else if p.at(SK::Arrow) {
      let entered = p.precede(exited);
      p.bump();
      p.eat(SK::Ident);
      p.exit(entered, SK::ArrowExpr)
    } else if p.at(SK::LSquare) {
      let entered = p.precede(exited);
      p.bump();
      expr(p, tds);
      p.eat(SK::RSquare);
      p.exit(entered, SK::SubscriptExpr)
    } else {
      break;
    };
  }
  Some(exited)
}

fn infix_prec(p: &mut Parser<'_, SK>) -> Option<u8> {
  if p.at(SK::Star) || p.at(SK::Slash) || p.at(SK::Percent) {
    Some(10)
  } else if p.at(SK::Plus) || p.at(SK::Minus) {
    Some(9)
  } else if p.at(SK::LtLt) || p.at(SK::GtGt) {
    Some(8)
  } else if p.at(SK::Lt) || p.at(SK::LtEq) || p.at(SK::Gt) || p.at(SK::GtEq) {
    Some(7)
  } else if p.at(SK::EqEq) || p.at(SK::BangEq) {
    Some(6)
  } else if p.at(SK::And) {
    Some(5)
  } else if p.at(SK::Carat) {
    Some(4)
  } else if p.at(SK::Bar) {
    Some(3)
  } else if p.at(SK::AndAnd) {
    Some(2)
  } else if p.at(SK::BarBar) {
    Some(1)
  } else {
    None
  }
}
