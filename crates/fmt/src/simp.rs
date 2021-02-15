use crate::expr::get as get_expr;
use crate::ty::get as get_ty;
use crate::util::Cx;
use syntax::ast::{IncDecKind, Simp};

#[must_use]
pub(crate) fn get(cx: &mut Cx, simp: Simp) -> Option<()> {
  match simp {
    Simp::AsgnSimp(simp) => {
      get_expr(cx, simp.lhs()?)?;
      cx.push(" = ");
      get_expr(cx, simp.rhs()?)?;
    }
    Simp::IncDecSimp(simp) => {
      get_expr(cx, simp.expr()?)?;
      let s = match simp.inc_dec()?.kind {
        IncDecKind::PlusPlus => "++",
        IncDecKind::MinusMinus => "--",
      };
      cx.push(s);
    }
    Simp::DeclSimp(simp) => {
      get_ty(cx, simp.ty()?)?;
      cx.push(" ");
      cx.push(simp.ident()?.text());
      if let Some(tail) = simp.defn_tail() {
        cx.push(" = ");
        get_expr(cx, tail.expr()?)?;
      }
    }
    Simp::ExprSimp(simp) => get_expr(cx, simp.expr()?)?,
    Simp::AmbiguousSimp(simp) => {
      // we don't have statics information right now. if we did, we would format
      // the expression form as `lhs * rhs` and the declaration form as `lhs*
      // rhs`. let's assume the declaration form is far more common, since the
      // expression form basically should never appear in a real program. (why
      // would you multiply two numbers and throw the result away?)
      cx.push(simp.lhs()?.text());
      cx.push("* ");
      cx.push(simp.rhs()?.text());
    }
  }
  Some(())
}
