use crate::util::name::Name;
use crate::util::ty::{Ty, TyDb};
use crate::util::{unify, ItemDb, NameToTy};
use crate::{expr, ty};
use syntax::ast::{AsgnOp, BlockStmt, Expr, Simp, Stmt, UnOp};

pub fn get_block(
  items: &ItemDb,
  vars: &mut NameToTy,
  tys: &mut TyDb,
  ret_ty: Ty,
  block: BlockStmt,
) -> Option<bool> {
  let mut end = false;
  for stmt in block.stmts() {
    if end {
      // unreachable
      return None;
    }
    end = get(items, vars, tys, ret_ty, stmt)?;
  }
  Some(end)
}

fn get(
  items: &ItemDb,
  vars: &mut NameToTy,
  tys: &mut TyDb,
  ret_ty: Ty,
  stmt: Stmt,
) -> Option<bool> {
  match stmt {
    Stmt::SimpStmt(stmt) => {
      get_simp(items, vars, tys, stmt.simp()?)?;
      Some(false)
    }
    Stmt::IfStmt(stmt) => {
      let cond_ty = expr::get(items, vars, tys, stmt.cond()?)?;
      unify(tys, Ty::Bool, cond_ty)?;
      let if_end = get(items, &mut vars.clone(), tys, ret_ty, stmt.yes()?)?;
      let else_end = match stmt.no() {
        None => false,
        Some(no) => get(items, &mut vars.clone(), tys, ret_ty, no.stmt()?)?,
      };
      Some(if_end && else_end)
    }
    Stmt::WhileStmt(stmt) => {
      let cond_ty = expr::get(items, vars, tys, stmt.cond()?)?;
      unify(tys, Ty::Bool, cond_ty)?;
      get(items, &mut vars.clone(), tys, ret_ty, stmt.body()?)?;
      Some(false)
    }
    Stmt::ForStmt(stmt) => {
      let mut vars = vars.clone();
      get_simp(items, &mut vars, tys, stmt.init()?)?;
      let cond_ty = expr::get(items, &vars, tys, stmt.cond()?)?;
      unify(tys, Ty::Bool, cond_ty)?;
      let step = stmt.step()?;
      if let Simp::DeclSimp(_) | Simp::DefnSimp(_) = step {
        return None;
      }
      get_simp(items, &mut vars, tys, step)?;
      get(items, &mut vars, tys, ret_ty, stmt.body()?)?;
      Some(false)
    }
    Stmt::ReturnStmt(stmt) => {
      match (stmt.expr(), ret_ty == Ty::Void) {
        (Some(_), true) | (None, false) => return None,
        (Some(expr), false) => {
          let ty = expr::get(items, vars, tys, expr)?;
          unify(tys, ret_ty, ty)?;
        }
        (None, true) => {}
      }
      Some(true)
    }
    Stmt::BlockStmt(stmt) => get_block(items, vars, tys, ret_ty, stmt),
    Stmt::AssertStmt(stmt) => {
      let ty = expr::get(items, vars, tys, stmt.expr()?)?;
      unify(tys, Ty::Bool, ty)?;
      Some(false)
    }
    Stmt::ErrorStmt(stmt) => {
      let ty = expr::get(items, vars, tys, stmt.expr()?)?;
      unify(tys, Ty::String, ty)?;
      Some(false)
    }
  }
}

fn get_simp(
  items: &ItemDb,
  vars: &mut NameToTy,
  tys: &mut TyDb,
  simp: Simp,
) -> Option<()> {
  match simp {
    Simp::AsgnSimp(simp) => {
      let lhs = simp.lhs()?;
      if !is_lv(&lhs)? {
        return None;
      }
      let lhs_ty = expr::get(items, vars, tys, lhs)?;
      let rhs_ty = expr::get(items, vars, tys, simp.rhs()?)?;
      let want_rhs_ty = asgn_op_ty(tys, lhs_ty, simp.op()?)?;
      unify(tys, want_rhs_ty, rhs_ty)?;
    }
    Simp::IncSimp(simp) => {
      get_inc_dec(items, vars, tys, simp.expr()?)?;
    }
    Simp::DecSimp(simp) => {
      get_inc_dec(items, vars, tys, simp.expr()?)?;
    }
    Simp::DeclSimp(simp) => {
      let ty = ty::get(&items.type_defs, tys, simp.ty()?)?;
      if vars.insert(Name::new(simp.ident()?.text()), ty).is_some() {
        return None;
      }
    }
    Simp::DefnSimp(simp) => {
      let ty = ty::get(&items.type_defs, tys, simp.ty()?)?;
      let expr_ty = expr::get(items, vars, tys, simp.expr()?)?;
      unify(tys, ty, expr_ty)?;
      if vars.insert(Name::new(simp.ident()?.text()), ty).is_some() {
        return None;
      }
    }
    Simp::ExprSimp(simp) => {
      expr::get(items, vars, tys, simp.expr()?)?;
    }
  }
  Some(())
}

fn get_inc_dec(
  items: &ItemDb,
  vars: &NameToTy,
  tys: &mut TyDb,
  expr: Expr,
) -> Option<()> {
  if !is_lv(&expr)? {
    return None;
  }
  let ty = expr::get(items, vars, tys, expr)?;
  unify(tys, Ty::Int, ty)?;
  Some(())
}

fn is_lv(expr: &Expr) -> Option<bool> {
  match expr {
    Expr::IdentExpr(_) => Some(true),
    Expr::ParenExpr(expr) => is_lv(&expr.expr()?),
    Expr::UnOpExpr(expr) => match expr.op()? {
      UnOp::Star(_) => is_lv(&expr.expr()?),
      UnOp::Bang(_) | UnOp::Tilde(_) | UnOp::Minus(_) => Some(false),
    },
    Expr::DotExpr(expr) => is_lv(&expr.expr()?),
    Expr::ArrowExpr(expr) => is_lv(&expr.expr()?),
    Expr::SubscriptExpr(expr) => is_lv(&expr.array()?),
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
    | Expr::AllocArrayExpr(_) => Some(false),
  }
}

fn asgn_op_ty(tys: &mut TyDb, lhs_ty: Ty, op: AsgnOp) -> Option<Ty> {
  match op {
    AsgnOp::Eq(_) => Some(lhs_ty),
    AsgnOp::PlusEq(_)
    | AsgnOp::MinusEq(_)
    | AsgnOp::StarEq(_)
    | AsgnOp::SlashEq(_)
    | AsgnOp::PercentEq(_)
    | AsgnOp::LtLtEq(_)
    | AsgnOp::GtGtEq(_)
    | AsgnOp::AndEq(_)
    | AsgnOp::CaratEq(_)
    | AsgnOp::BarEq(_) => {
      unify(tys, Ty::Int, lhs_ty)?;
      Some(Ty::Int)
    }
  }
}
