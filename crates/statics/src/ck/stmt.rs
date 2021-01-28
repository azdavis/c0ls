use crate::error::{Assignment, ErrorKind};
use crate::ty::Ty;
use crate::util::{add_var, unify, Cx, ItemDb, VarDb};
use syntax::ast::{
  AsgnOp, AsgnOpKind, BlockStmt, Expr, IncDecKind, Simp, Stmt, Syntax, UnOpKind,
};
use syntax::rowan::TextRange;
use unwrap_or::unwrap_or;

pub(crate) fn get_block(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &mut VarDb,
  ret_ty: Ty,
  block: BlockStmt,
) -> bool {
  let mut end = false;
  let mut reported = false;
  for stmt in block.stmts() {
    if end && !reported {
      cx.error(stmt.syntax().text_range(), ErrorKind::Unreachable);
      reported = true;
    }
    if get(cx, items, vars, ret_ty, stmt) {
      end = true;
    }
  }
  end
}

fn get(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &mut VarDb,
  ret_ty: Ty,
  stmt: Stmt,
) -> bool {
  match stmt {
    Stmt::SimpStmt(stmt) => {
      get_simp(cx, items, vars, stmt.simp());
      false
    }
    Stmt::IfStmt(stmt) => {
      let cond_ty = super::expr::get_opt(cx, items, vars, stmt.cond());
      unify(cx, Ty::Bool, cond_ty);
      let if_end = get_opt_or(cx, items, &mut vars.clone(), ret_ty, stmt.yes());
      let else_end = match stmt.no() {
        None => false,
        Some(no) => get_opt_or(cx, items, &mut vars.clone(), ret_ty, no.stmt()),
      };
      if_end && else_end
    }
    Stmt::WhileStmt(stmt) => {
      let cond_ty = super::expr::get_opt(cx, items, vars, stmt.cond());
      unify(cx, Ty::Bool, cond_ty);
      get_opt_or(cx, items, &mut vars.clone(), ret_ty, stmt.body());
      false
    }
    Stmt::ForStmt(stmt) => {
      let mut vars = vars.clone();
      get_simp(cx, items, &mut vars, stmt.init());
      let cond_ty = super::expr::get_opt(cx, items, &vars, stmt.cond());
      unify(cx, Ty::Bool, cond_ty);
      if let Some(step) = stmt.step() {
        if let Simp::DeclSimp(ref decl) = step {
          cx.error(decl.syntax().text_range(), ErrorKind::InvalidStepDecl);
        }
        get_simp(cx, items, &mut vars, Some(step));
      }
      get_opt_or(cx, items, &mut vars, ret_ty, stmt.body());
      false
    }
    Stmt::ReturnStmt(stmt) => {
      let ty = super::expr::get_opt(cx, items, vars, stmt.expr());
      match (ty, ret_ty == Ty::Void) {
        (Some((range, _)), true) => cx.error(range, ErrorKind::ReturnExprVoid),
        (None, false) => {
          cx.error(stmt.syntax().text_range(), ErrorKind::NoReturnExprNotVoid)
        }
        (Some(_), false) => {
          unify(cx, ret_ty, ty);
        }
        (None, true) => {}
      }
      true
    }
    Stmt::BlockStmt(stmt) => get_block(cx, items, vars, ret_ty, stmt),
    Stmt::AssertStmt(stmt) => {
      let ty = super::expr::get_opt(cx, items, vars, stmt.expr());
      unify(cx, Ty::Bool, ty);
      false
    }
    Stmt::ErrorStmt(stmt) => {
      let ty = super::expr::get_opt(cx, items, vars, stmt.expr());
      unify(cx, Ty::String, ty);
      false
    }
  }
}

/// does NOT report an error if it is None, so only call this with optional
/// things from the AST (that have a corresponding parse error).
fn get_opt_or(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &mut VarDb,
  ret_ty: Ty,
  stmt: Option<Stmt>,
) -> bool {
  stmt.map_or(false, |stmt| get(cx, items, vars, ret_ty, stmt))
}

fn get_simp(cx: &mut Cx, items: &ItemDb, vars: &mut VarDb, simp: Option<Simp>) {
  let simp = unwrap_or!(simp, return);
  match simp {
    Simp::AsgnSimp(simp) => {
      let lhs = simp.lhs();
      if let Some(ref lhs) = lhs {
        if !is_lv(lhs) {
          cx.error(
            lhs.syntax().text_range(),
            ErrorKind::CannotAssign(Assignment::Assign),
          );
        }
      }
      let lhs_ty = super::expr::get_opt(cx, items, vars, lhs);
      let rhs_ty = super::expr::get_opt(cx, items, vars, simp.rhs());
      let want_rhs_ty = asgn_op_ty(cx, lhs_ty, simp.op());
      unify(cx, want_rhs_ty, rhs_ty);
    }
    Simp::IncDecSimp(simp) => {
      let expr = simp.expr();
      if let Some(ref expr) = expr {
        if !is_lv(expr) {
          let assign = match simp.inc_dec() {
            Some(inc_dec) => match inc_dec.kind {
              IncDecKind::PlusPlus => Assignment::Inc,
              IncDecKind::MinusMinus => Assignment::Dec,
            },
            // this really shouldn't happen.
            None => Assignment::Assign,
          };
          cx.error(expr.syntax().text_range(), ErrorKind::CannotAssign(assign));
        }
      }
      let ty = super::expr::get_opt(cx, items, vars, expr);
      unify(cx, Ty::Int, ty);
    }
    Simp::DeclSimp(simp) => {
      let ty = super::ty::get_opt(cx, &items.type_defs, simp.ty());
      let defined = match simp.defn_tail() {
        None => false,
        Some(defn_tail) => {
          let expr_ty = super::expr::get_opt(cx, items, vars, defn_tail.expr());
          if let Some((_, ty)) = ty {
            unify(cx, ty, expr_ty);
          }
          true
        }
      };
      if let (Some(ident), Some((ty_range, ty))) = (simp.ident(), ty) {
        add_var(cx, vars, ident, ty_range, ty, defined);
      }
    }
    Simp::ExprSimp(simp) => {
      super::expr::get_opt(cx, items, vars, simp.expr());
    }
  }
}

fn is_lv(expr: &Expr) -> bool {
  match expr {
    Expr::IdentExpr(_) => true,
    Expr::ParenExpr(expr) => is_lv_opt(&expr.expr()),
    Expr::UnOpExpr(expr) => match unwrap_or!(expr.op(), return true).kind {
      UnOpKind::Star => is_lv_opt(&expr.expr()),
      UnOpKind::Bang | UnOpKind::Tilde | UnOpKind::Minus => false,
    },
    Expr::DotExpr(expr) => is_lv_opt(&expr.expr()),
    Expr::ArrowExpr(expr) => is_lv_opt(&expr.expr()),
    Expr::SubscriptExpr(expr) => is_lv_opt(&expr.array()),
    Expr::DecExpr(_)
    | Expr::HexExpr(_)
    | Expr::StringExpr(_)
    | Expr::CharExpr(_)
    | Expr::TrueExpr(_)
    | Expr::FalseExpr(_)
    | Expr::NullExpr(_)
    | Expr::BinOpExpr(_)
    | Expr::TernaryExpr(_)
    | Expr::CallExpr(_)
    | Expr::AllocExpr(_)
    | Expr::AllocArrayExpr(_) => false,
  }
}

fn is_lv_opt(expr: &Option<Expr>) -> bool {
  expr.as_ref().map_or(true, is_lv)
}

fn asgn_op_ty(
  cx: &mut Cx,
  lhs_ty: Option<(TextRange, Ty)>,
  op: Option<AsgnOp>,
) -> Ty {
  match unwrap_or!(op, return Ty::Error).kind {
    AsgnOpKind::Eq => lhs_ty.map_or(Ty::Error, |x| x.1),
    AsgnOpKind::PlusEq
    | AsgnOpKind::MinusEq
    | AsgnOpKind::StarEq
    | AsgnOpKind::SlashEq
    | AsgnOpKind::PercentEq
    | AsgnOpKind::LtLtEq
    | AsgnOpKind::GtGtEq
    | AsgnOpKind::AndEq
    | AsgnOpKind::CaratEq
    | AsgnOpKind::BarEq => {
      unify(cx, Ty::Int, lhs_ty);
      Ty::Int
    }
  }
}
