use crate::expr::get as get_expr;
use crate::ty::get as get_ty;
use crate::util::Cx;
use ast_ptr::AstPtr;
use hir::{AssignOp, IncDec, MathOp, Name};
use syntax::ast::{AsgnOpKind, IncDecKind, Simp};

pub(crate) fn get(cx: &mut Cx, simp: Simp) -> Option<hir::SimpId> {
  let ptr = AstPtr::new(&simp);
  let data = match simp {
    Simp::AsgnSimp(simp) => {
      let lhs = get_expr(cx, simp.lhs());
      let op = asgn_op(simp.op()?.kind);
      let rhs = get_expr(cx, simp.rhs());
      hir::Simp::Assign(lhs, op, rhs)
    }
    Simp::IncDecSimp(simp) => {
      let kind = match simp.inc_dec()?.kind {
        IncDecKind::PlusPlus => IncDec::Inc,
        IncDecKind::MinusMinus => IncDec::Dec,
      };
      let expr = get_expr(cx, simp.expr());
      hir::Simp::IncDec(expr, kind)
    }
    Simp::DeclSimp(simp) => {
      let name: Name = simp.ident()?.text().into();
      let ty = get_ty(cx, simp.ty());
      let expr = simp.defn_tail().map(|defn| get_expr(cx, defn.expr()));
      hir::Simp::Decl(name, ty, expr)
    }
    Simp::ExprSimp(simp) => hir::Simp::Expr(get_expr(cx, simp.expr())),
    Simp::AmbiguousSimp(simp) => {
      let lhs: Name = simp.lhs()?.text().into();
      let rhs: Name = simp.rhs()?.text().into();
      hir::Simp::Ambiguous(lhs, rhs)
    }
  };
  let ret = cx.arenas.simp.alloc(data);
  cx.ptrs.simp.insert(ptr, ret);
  cx.ptrs.simp_back.insert(ret, ptr);
  Some(ret)
}

fn asgn_op(op: AsgnOpKind) -> AssignOp {
  match op {
    AsgnOpKind::Eq => AssignOp::Eq,
    AsgnOpKind::PlusEq => AssignOp::OpEq(MathOp::Add),
    AsgnOpKind::MinusEq => AssignOp::OpEq(MathOp::Sub),
    AsgnOpKind::StarEq => AssignOp::OpEq(MathOp::Mul),
    AsgnOpKind::SlashEq => AssignOp::OpEq(MathOp::Div),
    AsgnOpKind::PercentEq => AssignOp::OpEq(MathOp::Mod),
    AsgnOpKind::LtLtEq => AssignOp::OpEq(MathOp::Shl),
    AsgnOpKind::GtGtEq => AssignOp::OpEq(MathOp::Shr),
    AsgnOpKind::AndEq => AssignOp::OpEq(MathOp::BitAnd),
    AsgnOpKind::CaratEq => AssignOp::OpEq(MathOp::BitXor),
    AsgnOpKind::BarEq => AssignOp::OpEq(MathOp::BitOr),
  }
}
