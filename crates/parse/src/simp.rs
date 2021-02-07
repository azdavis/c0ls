use crate::expr::{expr, expr_opt};
use crate::ty::ty_opt;
use crate::util::TypeDefs;
use syntax::event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) fn simp_opt(
  p: &mut Parser<'_, SK>,
  tds: &TypeDefs,
) -> Option<Exited> {
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
