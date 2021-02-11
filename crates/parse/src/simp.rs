use crate::expr::{expr, expr_opt};
use crate::ty::ty_opt;
use syntax::event_parse::{Exited, Parser};
use syntax::SyntaxKind as SK;

pub(crate) fn simp_opt(p: &mut Parser<'_, SK>) -> Option<Exited> {
  let mut could_be_ty = true;
  if p.at(SK::Ident) {
    match p.peek_n(1).map(|tok| tok.kind) {
      // - `foo<EOF>` is definitely a parse error.
      // - `foo(` is definitely the beginning of a function call.
      // - `foo;` is definitely an identifier expr stmt.
      //
      // but if ty_opt sees an ident (which it will, because we are at one) it
      // will consume it. stop that preemptively by not calling it.
      None | Some(SK::LRound) | Some(SK::Semicolon) => could_be_ty = false,
      Some(SK::Star) => {
        if p.peek_n(2).map_or(false, |tok| tok.kind == SK::Ident)
          && p.peek_n(3).map_or(false, |tok| tok.kind == SK::Semicolon)
        {
          // `foo * bar;` is ambiguous. it might be declaring bar of type
          // pointer-to-foo if foo is a typedef in scope, or it might be
          // multiplying foo and bar and discarding the result. we can't know
          // for sure without semantic information (i.e. what typedefs are in
          // scope right now). but we don't want to have to keep track of that
          // while parsing:
          //
          // - what if we want to parse each file in parallel? now we can't
          //   because we need to do name resolution, which can happen across
          //   files.
          // - we're going to check the statics of the parse tree later. we
          //   don't want to things related to that now.
          //
          // so, since it's ambiguous, we emit a special construct and check it
          // later in statics.
          let entered = p.enter();
          p.eat(SK::Ident);
          p.eat(SK::Star);
          p.eat(SK::Ident);
          return Some(p.exit(entered, SK::AmbiguousSimp));
        }
        // else, might be `foo ** bar;`, `foo*[] bar`, etc
      }
      // else, might be `foo[] bar;`, `foo bar;`, etc
      _ => {}
    }
  }
  if could_be_ty {
    if let Some(ty) = ty_opt(p) {
      let entered = p.precede(ty);
      p.eat(SK::Ident);
      if p.at(SK::Eq) {
        let entered = p.enter();
        p.bump();
        expr(p);
        p.exit(entered, SK::DefnTail);
      }
      return Some(p.exit(entered, SK::DeclSimp));
    }
  }
  let exited = expr_opt(p)?;
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
    expr(p);
    p.exit(entered, SK::AsgnSimp)
  } else if p.at(SK::PlusPlus) || p.at(SK::MinusMinus) {
    p.bump();
    p.exit(entered, SK::IncDecSimp)
  } else {
    p.exit(entered, SK::ExprSimp)
  };
  Some(completed)
}
