use crate::ptr::AstPtr;
use crate::ty::get as get_ty;
use crate::util::Cx;
use hir::{BinOp, MathOp, UnOp};
use syntax::ast::{BinOpKind, Expr, UnOpKind};
use unwrap_or::unwrap_or;

pub(crate) fn get(cx: &mut Cx, expr: Option<Expr>) -> hir::ExprId {
  let (ptr, data) = expr.map_or((None, hir::Expr::None), |expr| {
    (Some(AstPtr::new(&expr)), get_impl(cx, expr))
  });
  let ret = cx.arenas.expr.alloc(data);
  if let Some(ptr) = ptr {
    cx.ptrs.expr.insert(ptr, ret);
    cx.ptrs.expr_back.insert(ret, ptr);
  }
  ret
}

fn get_impl(cx: &mut Cx, expr: Expr) -> hir::Expr {
  match expr {
    Expr::DecExpr(_) | Expr::HexExpr(_) => hir::Expr::Int,
    Expr::StringExpr(_) => hir::Expr::String,
    Expr::CharExpr(_) => hir::Expr::Char,
    Expr::TrueExpr(_) | Expr::FalseExpr(_) => hir::Expr::Bool,
    Expr::NullExpr(_) => hir::Expr::Null,
    Expr::IdentExpr(expr) => {
      let name = unwrap_or!(expr.ident(), return hir::Expr::None);
      hir::Expr::Name(name.text().into())
    }
    Expr::ParenExpr(expr) => {
      let expr = unwrap_or!(expr.expr(), return hir::Expr::None);
      get_impl(cx, expr)
    }
    Expr::BinOpExpr(expr) => {
      let op = unwrap_or!(expr.op(), return hir::Expr::None);
      let op = bin_op(op.kind);
      let lhs = get(cx, expr.lhs());
      let rhs = get(cx, expr.rhs());
      hir::Expr::BinOp(lhs, op, rhs)
    }
    Expr::UnOpExpr(expr) => {
      let op = unwrap_or!(expr.op(), return hir::Expr::None);
      let op = un_op(op.kind);
      let expr = get(cx, expr.expr());
      hir::Expr::UnOp(op, expr)
    }
    Expr::TernaryExpr(expr) => {
      let cond = get(cx, expr.cond());
      let yes = get(cx, expr.yes());
      let no = get(cx, expr.no());
      hir::Expr::Ternary(cond, yes, no)
    }
    Expr::CallExpr(expr) => {
      let name = unwrap_or!(expr.ident(), return hir::Expr::None);
      let args: Vec<_> = expr.args().map(|arg| get(cx, arg.expr())).collect();
      hir::Expr::Call(name.text().into(), args)
    }
    Expr::DotExpr(expr) => {
      let field = unwrap_or!(expr.ident(), return hir::Expr::None);
      let expr = get(cx, expr.expr());
      hir::Expr::Dot(expr, field.text().into())
    }
    Expr::ArrowExpr(ref inner) => {
      let field = unwrap_or!(inner.ident(), return hir::Expr::None);
      let ptr = AstPtr::new(&expr);
      let expr = get(cx, inner.expr());
      let deref = cx.arenas.expr.alloc(hir::Expr::UnOp(UnOp::Deref, expr));
      // no entry for `cx.ptrs.expr`, since we'll have an entry from expr to the
      // id of the Dot
      cx.ptrs.expr_back.insert(deref, ptr);
      hir::Expr::Dot(deref, field.text().into())
    }
    Expr::SubscriptExpr(expr) => {
      let array = get(cx, expr.array());
      let idx = get(cx, expr.idx());
      hir::Expr::Subscript(array, idx)
    }
    Expr::AllocExpr(expr) => {
      let ty = get_ty(cx, expr.ty());
      hir::Expr::Alloc(ty)
    }
    Expr::AllocArrayExpr(expr) => {
      let ty = get_ty(cx, expr.ty());
      let expr = get(cx, expr.expr());
      hir::Expr::AllocArray(ty, expr)
    }
  }
}

fn bin_op(op: BinOpKind) -> BinOp {
  match op {
    BinOpKind::Plus => BinOp::Math(MathOp::Add),
    BinOpKind::Minus => BinOp::Math(MathOp::Sub),
    BinOpKind::Star => BinOp::Math(MathOp::Mul),
    BinOpKind::Slash => BinOp::Math(MathOp::Div),
    BinOpKind::Percent => BinOp::Math(MathOp::Mod),
    BinOpKind::LtLt => BinOp::Math(MathOp::Shl),
    BinOpKind::GtGt => BinOp::Math(MathOp::Shr),
    BinOpKind::And => BinOp::Math(MathOp::BitAnd),
    BinOpKind::Carat => BinOp::Math(MathOp::BitXor),
    BinOpKind::Bar => BinOp::Math(MathOp::BitOr),
    BinOpKind::EqEq => BinOp::Eq,
    BinOpKind::BangEq => BinOp::Neq,
    BinOpKind::Lt => BinOp::Lt,
    BinOpKind::LtEq => BinOp::LtEq,
    BinOpKind::Gt => BinOp::Gt,
    BinOpKind::GtEq => BinOp::GtEq,
    BinOpKind::AndAnd => BinOp::And,
    BinOpKind::BarBar => BinOp::Or,
  }
}

fn un_op(op: UnOpKind) -> UnOp {
  match op {
    UnOpKind::Bang => UnOp::Not,
    UnOpKind::Tilde => UnOp::BitNot,
    UnOpKind::Minus => UnOp::Neg,
    UnOpKind::Star => UnOp::Deref,
  }
}
