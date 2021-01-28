use crate::error::{Assignment, ErrorKind};
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
      let mut if_vars = vars.clone();
      let if_end = get_opt_or(cx, items, &mut if_vars, ret_ty, stmt.yes());
      let else_end = match stmt.no() {
        None => false,
        Some(no) => {
          let mut else_vars = vars.clone();
          let ret = get_opt_or(cx, items, &mut else_vars, ret_ty, no.stmt());
          for (name, data) in vars.iter_mut() {
            let defined = if_vars[name].defined && else_vars[name].defined;
            assert!(!data.defined || defined);
            data.defined = defined;
          }
          ret
        }
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
      let mut for_vars = vars.clone();
      if let Some(var) = get_simp(cx, items, &mut for_vars, stmt.init()) {
        vars.get_mut(var.text()).unwrap().defined = true;
      }
      let cond_ty = super::expr::get_opt(cx, items, &for_vars, stmt.cond());
      unify(cx, Ty::Bool, cond_ty);
      if let Some(step) = stmt.step() {
        if let Simp::DeclSimp(ref decl) = step {
          cx.error(decl.syntax().text_range(), ErrorKind::InvalidStepDecl);
        }
        get_simp(cx, items, &mut for_vars, Some(step));
      }
      get_opt_or(cx, items, &mut for_vars, ret_ty, stmt.body());
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
    Stmt::BlockStmt(stmt) => {
      let mut block_vars = vars.clone();
      let ret = get_block(cx, items, &mut block_vars, ret_ty, stmt);
      for (name, data) in vars.iter_mut() {
        let defined = block_vars[name].defined;
        assert!(!data.defined || defined);
        data.defined = defined;
      }
      ret
    }
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
      let rhs_ty = super::expr::get_opt(cx, items, vars, simp.rhs());
      let lhs = simp.lhs();
      let var = lv_var(cx, Assignment::Assign, &lhs);
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
      let lhs_ty = super::expr::get_opt(cx, items, vars, lhs);
      if let Some(want_lhs_ty) = want_lhs_ty {
        unify(cx, want_lhs_ty, lhs_ty);
      }
    }
    Simp::IncDecSimp(simp) => {
      let asgn = match simp.inc_dec() {
        Some(inc_dec) => match inc_dec.kind {
          IncDecKind::PlusPlus => Assignment::Inc,
          IncDecKind::MinusMinus => Assignment::Dec,
        },
        // this really shouldn't happen.
        None => Assignment::Assign,
      };
      let expr = simp.expr();
      // get the error if any, but ignore the var (this doesn't init it).
      lv_var(cx, asgn, &expr);
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
  ret
}

fn lv_var(
  cx: &mut Cx,
  asgn: Assignment,
  expr: &Option<Expr>,
) -> Option<SyntaxToken> {
  let expr = expr.as_ref()?;
  match lv(expr) {
    None => {
      cx.error(expr.syntax().text_range(), ErrorKind::CannotAssign(asgn));
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
    Expr::UnOpExpr(expr) => match unwrap_or!(expr.op(), return None).kind {
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
  expr.as_ref().and_then(lv).map(|_| Lv::Other)
}
