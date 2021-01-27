use crate::util::error::{ErrorKind, Thing};
use crate::util::name::Name;
use crate::util::ty::Ty;
use crate::util::{unify, Cx, ItemDb, NameToTy};
use crate::{expr, ty};
use std::collections::hash_map::Entry;
use syntax::ast::{AsgnOp, AsgnOpKind, BlockStmt, Expr, Simp, Stmt, UnOpKind};

pub(crate) fn get_block(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &mut NameToTy,
  ret_ty: Ty,
  block: BlockStmt,
) -> bool {
  let mut end = false;
  for stmt in block.stmts() {
    if end {
      todo!("unreachable");
    }
    end = get(cx, items, vars, ret_ty, stmt);
  }
  end
}

fn get(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &mut NameToTy,
  ret_ty: Ty,
  stmt: Stmt,
) -> bool {
  match stmt {
    Stmt::SimpStmt(stmt) => {
      get_simp(cx, items, vars, stmt.simp());
      false
    }
    Stmt::IfStmt(stmt) => {
      let cond_ty = expr::get_opt(cx, items, vars, stmt.cond());
      unify(cx, Ty::Bool, cond_ty);
      let if_end = get_opt(cx, items, &mut vars.clone(), ret_ty, stmt.yes());
      let else_end = match stmt.no() {
        None => false,
        Some(no) => get_opt(cx, items, &mut vars.clone(), ret_ty, no.stmt()),
      };
      if_end && else_end
    }
    Stmt::WhileStmt(stmt) => {
      let cond_ty = expr::get_opt(cx, items, vars, stmt.cond());
      unify(cx, Ty::Bool, cond_ty);
      get_opt(cx, items, &mut vars.clone(), ret_ty, stmt.body());
      false
    }
    Stmt::ForStmt(stmt) => {
      let mut vars = vars.clone();
      get_simp(cx, items, &mut vars, stmt.init());
      let cond_ty = expr::get_opt(cx, items, &vars, stmt.cond());
      unify(cx, Ty::Bool, cond_ty);
      if let Some(step) = stmt.step() {
        if let Simp::DeclSimp(_) = step {
          todo!("step cannot be decl")
        }
        get_simp(cx, items, &mut vars, Some(step));
      }
      get_opt(cx, items, &mut vars, ret_ty, stmt.body());
      false
    }
    Stmt::ReturnStmt(stmt) => {
      match (stmt.expr(), ret_ty == Ty::Void) {
        (Some(_), true) => todo!("return expr but void type"),
        (None, false) => todo!("no return expr but non-void type"),
        (Some(expr), false) => {
          let ty = expr::get(cx, items, vars, expr);
          unify(cx, ret_ty, ty);
        }
        (None, true) => {}
      }
      true
    }
    Stmt::BlockStmt(stmt) => get_block(cx, items, vars, ret_ty, stmt),
    Stmt::AssertStmt(stmt) => {
      let ty = expr::get_opt(cx, items, vars, stmt.expr());
      unify(cx, Ty::Bool, ty);
      false
    }
    Stmt::ErrorStmt(stmt) => {
      let ty = expr::get_opt(cx, items, vars, stmt.expr());
      unify(cx, Ty::String, ty);
      false
    }
  }
}

/// does NOT report an error if it is None, so only call this with optional
/// things from the AST (that have a corresponding parse error).
fn get_opt(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &mut NameToTy,
  ret_ty: Ty,
  stmt: Option<Stmt>,
) -> bool {
  stmt.map_or(false, |stmt| get(cx, items, vars, ret_ty, stmt))
}

fn get_simp(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &mut NameToTy,
  simp: Option<Simp>,
) {
  let simp = unwrap_or!(simp, return);
  match simp {
    Simp::AsgnSimp(simp) => {
      let lhs = simp.lhs();
      if !is_lv(&lhs) {
        todo!("cannot assign to expression");
      }
      let lhs_ty = expr::get_opt(cx, items, vars, lhs);
      let rhs_ty = expr::get_opt(cx, items, vars, simp.rhs());
      let want_rhs_ty = asgn_op_ty(cx, lhs_ty, simp.op());
      unify(cx, want_rhs_ty, rhs_ty);
    }
    Simp::IncDecSimp(simp) => {
      let expr = simp.expr();
      if !is_lv(&expr) {
        todo!("cannot inc/dec expression");
      }
      let ty = expr::get_opt(cx, items, vars, expr);
      unify(cx, Ty::Int, ty);
    }
    Simp::DeclSimp(simp) => {
      let ty = ty::get_opt(cx, &items.type_defs, simp.ty());
      if let Some(defn_tail) = simp.defn_tail() {
        let expr_ty = expr::get_opt(cx, items, vars, defn_tail.expr());
        unify(cx, ty, expr_ty);
      }
      let ident = unwrap_or!(simp.ident(), return);
      match vars.entry(Name::new(ident.text())) {
        Entry::Occupied(_) => {
          cx.errors
            .push(ident.text_range(), ErrorKind::Duplicate(Thing::Variable));
        }
        Entry::Vacant(entry) => {
          entry.insert(ty);
        }
      }
    }
    Simp::ExprSimp(simp) => {
      expr::get_opt(cx, items, vars, simp.expr());
    }
  }
}

fn is_lv(expr: &Option<Expr>) -> bool {
  let expr = unwrap_or!(expr.as_ref(), return true);
  match expr {
    Expr::IdentExpr(_) => true,
    Expr::ParenExpr(expr) => is_lv(&expr.expr()),
    Expr::UnOpExpr(expr) => match unwrap_or!(expr.op(), return true).kind {
      UnOpKind::Star => is_lv(&expr.expr()),
      UnOpKind::Bang | UnOpKind::Tilde | UnOpKind::Minus => false,
    },
    Expr::DotExpr(expr) => is_lv(&expr.expr()),
    Expr::ArrowExpr(expr) => is_lv(&expr.expr()),
    Expr::SubscriptExpr(expr) => is_lv(&expr.array()),
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

fn asgn_op_ty(cx: &mut Cx, lhs_ty: Ty, op: Option<AsgnOp>) -> Ty {
  match unwrap_or!(op, return Ty::Error).kind {
    AsgnOpKind::Eq => lhs_ty,
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
