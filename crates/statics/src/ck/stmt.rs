use crate::error::{ErrorKind, IncDec};
use crate::name::Name;
use crate::ty::Ty;
use crate::util::{add_var, unify, Cx, ItemDb, VarDb};
use syntax::ast::{
  AsgnOpKind, BlockStmt, Expr, IncDecKind, Simp, Stmt, Syntax, UnOpKind,
};
use syntax::SyntaxToken;
use unwrap_or::unwrap_or;

pub(crate) fn get_block(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &mut VarDb,
  ret_ty: Ty,
  in_loop: bool,
  block: BlockStmt,
) -> bool {
  let mut end = false;
  #[cfg(feature = "unreachable")]
  let mut reported = false;
  for stmt in block.stmts() {
    #[cfg(feature = "unreachable")]
    if end && !reported {
      cx.error(stmt.syntax().text_range(), ErrorKind::Unreachable);
      reported = true;
    }
    if get(cx, items, vars, ret_ty, in_loop, stmt) {
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
  in_loop: bool,
  stmt: Stmt,
) -> bool {
  match stmt {
    Stmt::SimpStmt(stmt) => {
      get_simp(cx, items, vars, stmt.simp());
      false
    }
    Stmt::IfStmt(stmt) => {
      let cond_ty = super::expr::get(cx, items, vars, stmt.cond());
      unify(cx, Ty::Bool, cond_ty);
      let mut if_vars = vars.clone();
      let if_end =
        get_opt_or(cx, items, &mut if_vars, ret_ty, in_loop, stmt.yes());
      let else_end = match stmt.no() {
        None => false,
        Some(no) => {
          let mut else_vars = vars.clone();
          let ret =
            get_opt_or(cx, items, &mut else_vars, ret_ty, in_loop, no.stmt());
          define(vars, |name| {
            if_vars[name].defined && else_vars[name].defined
          });
          ret
        }
      };
      if_end && else_end
    }
    Stmt::WhileStmt(stmt) => {
      let cond_ty = super::expr::get(cx, items, vars, stmt.cond());
      unify(cx, Ty::Bool, cond_ty);
      get_opt_or(cx, items, &mut vars.clone(), ret_ty, true, stmt.body());
      false
    }
    Stmt::ForStmt(stmt) => {
      let mut body_vars = vars.clone();
      let init = stmt.init().and_then(|x| x.simp());
      if let Some(var) = get_simp(cx, items, &mut body_vars, init) {
        vars.get_mut(var.text()).unwrap().defined = true;
      }
      let cond_ty = super::expr::get(cx, items, &body_vars, stmt.cond());
      unify(cx, Ty::Bool, cond_ty);
      let mut step_vars = body_vars.clone();
      get_opt_or(cx, items, &mut body_vars, ret_ty, true, stmt.body());
      if let Some(step) = stmt.step().and_then(|x| x.simp()) {
        if let Simp::DeclSimp(ref decl) = step {
          cx.error(decl.syntax().text_range(), ErrorKind::InvalidStepDecl);
        }
        define(&mut step_vars, |name| body_vars[name].defined);
        get_simp(cx, items, &mut step_vars, Some(step));
      }
      false
    }
    Stmt::ReturnStmt(stmt) => {
      let ty = super::expr::get(cx, items, vars, stmt.expr());
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
      // demanded by the spec
      define(vars, |_| true);
      true
    }
    Stmt::BlockStmt(stmt) => {
      let mut block_vars = vars.clone();
      let ret = get_block(cx, items, &mut block_vars, ret_ty, in_loop, stmt);
      define(vars, |name| block_vars[name].defined);
      ret
    }
    Stmt::AssertStmt(stmt) => {
      let ty = super::expr::get(cx, items, vars, stmt.expr());
      unify(cx, Ty::Bool, ty);
      false
    }
    Stmt::ErrorStmt(stmt) => {
      let ty = super::expr::get(cx, items, vars, stmt.expr());
      unify(cx, Ty::String, ty);
      false
    }
    Stmt::BreakStmt(stmt) => {
      if !in_loop {
        cx.error(stmt.syntax().text_range(), ErrorKind::BreakOutsideLoop);
      }
      define(vars, |_| true);
      true
    }
    Stmt::ContinueStmt(stmt) => {
      if !in_loop {
        cx.error(stmt.syntax().text_range(), ErrorKind::ContinueOutsideLoop);
      }
      define(vars, |_| true);
      true
    }
  }
}

fn get_opt_or(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &mut VarDb,
  ret_ty: Ty,
  in_loop: bool,
  stmt: Option<Stmt>,
) -> bool {
  stmt.map_or(false, |stmt| get(cx, items, vars, ret_ty, in_loop, stmt))
}

/// returns the newly-defined but previously declared variable, if there was
/// one. this is used as an optimization for checking `for` loops.
fn get_simp(
  cx: &mut Cx,
  items: &ItemDb,
  vars: &mut VarDb,
  simp: Option<Simp>,
) -> Option<SyntaxToken> {
  let mut ret: Option<SyntaxToken> = None;
  let simp = unwrap_or!(simp, return ret);
  match simp {
    Simp::AsgnSimp(simp) => {
      let rhs_ty = super::expr::get(cx, items, vars, simp.rhs());
      let lhs = simp.lhs();
      let var = lv_var(cx, None, &lhs);
      let want_lhs_ty = match simp.op() {
        None => None,
        Some(op) => match op.kind {
          AsgnOpKind::Eq => {
            if let Some(var) = var {
              if let Some(data) = vars.get_mut(var.text()) {
                data.defined = true;
                ret = Some(var);
              }
            }
            rhs_ty.map(|x| x.1)
          }
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
            unify(cx, Ty::Int, rhs_ty);
            Some(Ty::Int)
          }
        },
      };
      let lhs_ty = super::expr::get(cx, items, vars, lhs);
      if let Some(want_lhs_ty) = want_lhs_ty {
        unify(cx, want_lhs_ty, lhs_ty);
      }
    }
    Simp::IncDecSimp(simp) => {
      let inc_dec = match simp.inc_dec().unwrap().kind {
        IncDecKind::PlusPlus => IncDec::Inc,
        IncDecKind::MinusMinus => IncDec::Dec,
      };
      let expr = simp.expr();
      let is_star = expr
        .as_ref()
        .and_then(|e| match e {
          Expr::UnOpExpr(e) => e.op(),
          _ => None,
        })
        .map_or(false, |op| matches!(op.kind, UnOpKind::Star));
      if is_star {
        cx.error(simp.syntax().text_range(), ErrorKind::DerefIncDec(inc_dec))
      }
      // get the error if any, but ignore the var (this doesn't init it).
      lv_var(cx, Some(inc_dec), &expr);
      let ty = super::expr::get(cx, items, vars, expr);
      unify(cx, Ty::Int, ty);
    }
    Simp::DeclSimp(simp) => {
      let ty = super::ty::get_opt(cx, &items.type_defs, simp.ty());
      let defined = match simp.defn_tail() {
        None => false,
        Some(defn_tail) => {
          let expr_ty = super::expr::get(cx, items, vars, defn_tail.expr());
          if let Some((_, ty)) = ty {
            unify(cx, ty, expr_ty);
          }
          true
        }
      };
      if let (Some(ident), Some((ty_range, ty))) = (simp.ident(), ty) {
        add_var(cx, vars, &items.type_defs, ident, ty_range, ty, defined);
      }
    }
    Simp::ExprSimp(simp) => {
      super::expr::get(cx, items, vars, simp.expr());
    }
  }
  ret
}

/// for each (name, data) in vars, sets data.defined = f(name). but it must be
/// that either data.defined was already true, or f(name) is true.
///
/// this is used for when vars contains exactly the variables in scope after
/// finishing processing a statement, and f contains information about what
/// variables in vars are now defined after processing that statement.
fn define<F>(vars: &mut VarDb, mut f: F)
where
  F: FnMut(&Name) -> bool,
{
  for (name, data) in vars.iter_mut() {
    let defined = f(name);
    assert!(!data.defined || defined);
    data.defined = defined;
  }
}

fn lv_var(
  cx: &mut Cx,
  asgn: Option<IncDec>,
  expr: &Option<Expr>,
) -> Option<SyntaxToken> {
  let expr = expr.as_ref()?;
  match lv(expr) {
    None => {
      cx.error(expr.syntax().text_range(), ErrorKind::InvalidAssign(asgn));
      None
    }
    Some(lv) => match lv {
      Lv::Var(var) => Some(var),
      Lv::Other => None,
    },
  }
}

enum Lv {
  Var(SyntaxToken),
  Other,
}

fn lv(expr: &Expr) -> Option<Lv> {
  match expr {
    Expr::IdentExpr(ident) => Some(ident.ident().map_or(Lv::Other, Lv::Var)),
    Expr::ParenExpr(expr) => lv_opt(&expr.expr()),
    Expr::UnOpExpr(expr) => match expr.op()?.kind {
      UnOpKind::Star => lv_opt_other(&expr.expr()),
      UnOpKind::Bang | UnOpKind::Tilde | UnOpKind::Minus => None,
    },
    Expr::DotExpr(expr) => lv_opt_other(&expr.expr()),
    Expr::ArrowExpr(expr) => lv_opt_other(&expr.expr()),
    Expr::SubscriptExpr(expr) => lv_opt_other(&expr.array()),
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
    | Expr::AllocArrayExpr(_) => None,
  }
}

fn lv_opt(expr: &Option<Expr>) -> Option<Lv> {
  expr.as_ref().and_then(lv)
}

fn lv_opt_other(expr: &Option<Expr>) -> Option<Lv> {
  lv_opt(expr).map(|_| Lv::Other)
}
