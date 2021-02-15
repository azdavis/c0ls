use crate::ty::get as get_ty;
use crate::util::Cx;
use syntax::ast::{BinOpKind, Expr, UnOpKind};

#[must_use]
pub(crate) fn get(cx: &mut Cx, expr: Expr) -> Option<()> {
  get_prec(cx, 0, expr)
}

#[must_use]
fn get_prec(cx: &mut Cx, min_prec: u8, expr: Expr) -> Option<()> {
  match expr {
    Expr::DecExpr(expr) => cx.push(expr.dec_lit()?.text()),
    Expr::HexExpr(expr) => cx.push(expr.hex_lit()?.text()),
    Expr::StringExpr(expr) => cx.push(expr.string_lit()?.text()),
    Expr::CharExpr(expr) => cx.push(expr.char_lit()?.text()),
    Expr::TrueExpr(_) => cx.push("true"),
    Expr::FalseExpr(_) => cx.push("false"),
    Expr::NullExpr(_) => cx.push("NULL"),
    Expr::IdentExpr(expr) => cx.push(expr.ident()?.text()),
    Expr::ParenExpr(expr) => get_prec(cx, min_prec, expr.expr()?)?,
    Expr::BinOpExpr(expr) => {
      let op = expr.op()?;
      let prec = bin_op_prec(&op.kind);
      if prec < min_prec {
        cx.push("(");
      }
      get_prec(cx, prec, expr.lhs()?)?;
      cx.push(" ");
      cx.push(bin_op_str(&op.kind));
      cx.push(" ");
      get_prec(cx, prec, expr.rhs()?)?;
      if prec < min_prec {
        cx.push(")");
      }
    }
    Expr::UnOpExpr(expr) => {
      if UN_OP_PREC < min_prec {
        cx.push("(");
      }
      cx.push(un_op_str(&expr.op()?.kind));
      get_prec(cx, UN_OP_PREC, expr.expr()?)?;
      if UN_OP_PREC < min_prec {
        cx.push(")");
      }
    }
    Expr::TernaryExpr(expr) => {
      if TERNARY_PREC <= min_prec {
        cx.push("(");
      }
      get_prec(cx, TERNARY_PREC, expr.cond()?)?;
      cx.push(" ? ");
      get(cx, expr.yes()?)?;
      cx.push(" : ");
      get(cx, expr.no()?)?;
      if TERNARY_PREC <= min_prec {
        cx.push(")");
      }
    }
    Expr::CallExpr(expr) => {
      cx.push(expr.ident()?.text());
      cx.push("(");
      let mut args = expr.args();
      if let Some(arg) = args.next() {
        get(cx, arg.expr()?)?;
      }
      for arg in args {
        cx.push(", ");
        get(cx, arg.expr()?)?;
      }
      cx.push(")");
    }
    Expr::DotExpr(expr) => {
      get_prec(cx, TOP_PREC, expr.expr()?)?;
      cx.push(".");
      cx.push(expr.ident()?.text());
    }
    Expr::ArrowExpr(expr) => {
      get_prec(cx, TOP_PREC, expr.expr()?)?;
      cx.push("->");
      cx.push(expr.ident()?.text());
    }
    Expr::SubscriptExpr(expr) => {
      get_prec(cx, TOP_PREC, expr.array()?)?;
      cx.push("[");
      get(cx, expr.idx()?)?;
      cx.push("]");
    }
    Expr::AllocExpr(expr) => {
      cx.push("alloc(");
      get_ty(cx, expr.ty()?)?;
      cx.push(")");
    }
    Expr::AllocArrayExpr(expr) => {
      cx.push("alloc_array(");
      get_ty(cx, expr.ty()?)?;
      cx.push(", ");
      get(cx, expr.expr()?)?;
      cx.push(")");
    }
  }
  Some(())
}

fn bin_op_str(op: &BinOpKind) -> &'static str {
  match *op {
    BinOpKind::Plus => "+",
    BinOpKind::Minus => "-",
    BinOpKind::Star => "*",
    BinOpKind::Slash => "/",
    BinOpKind::Percent => "%",
    BinOpKind::LtLt => "<<",
    BinOpKind::GtGt => ">>",
    BinOpKind::And => "&",
    BinOpKind::Carat => "^",
    BinOpKind::Bar => "|",
    BinOpKind::EqEq => "==",
    BinOpKind::BangEq => "!=",
    BinOpKind::Lt => "<",
    BinOpKind::LtEq => "<=",
    BinOpKind::Gt => ">",
    BinOpKind::GtEq => ">=",
    BinOpKind::AndAnd => "&&",
    BinOpKind::BarBar => "||",
  }
}

const TOP_PREC: u8 = 13;
const UN_OP_PREC: u8 = 12;

fn bin_op_prec(op: &BinOpKind) -> u8 {
  match *op {
    BinOpKind::Star | BinOpKind::Slash | BinOpKind::Percent => 11,
    BinOpKind::Plus | BinOpKind::Minus => 10,
    BinOpKind::LtLt | BinOpKind::GtGt => 9,
    BinOpKind::Lt | BinOpKind::LtEq | BinOpKind::Gt | BinOpKind::GtEq => 8,
    BinOpKind::EqEq | BinOpKind::BangEq => 7,
    BinOpKind::And => 6,
    BinOpKind::Carat => 5,
    BinOpKind::Bar => 4,
    BinOpKind::AndAnd => 3,
    BinOpKind::BarBar => 2,
  }
}

const TERNARY_PREC: u8 = 1;

fn un_op_str(op: &UnOpKind) -> &'static str {
  match *op {
    UnOpKind::Bang => "!",
    UnOpKind::Tilde => "~",
    UnOpKind::Minus => "-",
    UnOpKind::Star => "*",
  }
}
