use crate::expr::get as get_expr;
use crate::simp::{get as get_simp, VarInfo};
use crate::util::error::ErrorKind;
use crate::util::ty::Ty;
use crate::util::types::{Cx, Env, FnCx, Vars};
use crate::util::unify;
use hir::{Name, Stmt, StmtId};

pub(crate) fn get(
  cx: &mut Cx,
  env: &mut Env,
  fn_cx: &mut FnCx<'_>,
  in_loop: bool,
  stmt: StmtId,
) -> bool {
  match fn_cx.arenas.stmt[stmt] {
    Stmt::None => false,
    Stmt::Simp(simp) => {
      get_simp(cx, env, fn_cx, simp);
      false
    }
    Stmt::If(cond, yes, no) => {
      let cond_ty = get_expr(cx, env, fn_cx, cond);
      unify(cx, Ty::Bool, cond_ty, cond);
      let vars = fn_cx.vars.clone();
      let yes_diverges = get(cx, env, fn_cx, in_loop, yes);
      let yes_vars = std::mem::replace(&mut fn_cx.vars, vars);
      let no_diverges = match no {
        Some(no) => {
          let vars = fn_cx.vars.clone();
          let ret = get(cx, env, fn_cx, in_loop, no);
          let no_vars = std::mem::replace(&mut fn_cx.vars, vars);
          initialize(&mut fn_cx.vars, |name| {
            yes_vars[name].init && no_vars[name].init
          });
          ret
        }
        None => false,
      };
      yes_diverges && no_diverges
    }
    Stmt::While(cond, body) => {
      let cond_ty = get_expr(cx, env, fn_cx, cond);
      unify(cx, Ty::Bool, cond_ty, cond);
      let vars = fn_cx.vars.clone();
      get(cx, env, fn_cx, true, body);
      fn_cx.vars = vars;
      false
    }
    Stmt::For(init, cond, step, body) => {
      let mut vars = fn_cx.vars.clone();
      let init = init.map(|init| get_simp(cx, env, fn_cx, init));
      if let Some(VarInfo::Defn(name)) = init {
        vars.get_mut(name).unwrap().init = true;
      }
      let cond_ty = get_expr(cx, env, fn_cx, cond);
      unify(cx, Ty::Bool, cond_ty, cond);
      let mut step_vars = fn_cx.vars.clone();
      get(cx, env, fn_cx, true, body);
      if let Some(step) = step {
        initialize(&mut step_vars, |name| fn_cx.vars[name].init);
        fn_cx.vars = step_vars;
        let simp = get_simp(cx, env, fn_cx, step);
        if matches!(simp, VarInfo::Decl) {
          cx.err(step, ErrorKind::DeclInForStep);
        }
      }
      fn_cx.vars = vars;
      false
    }
    Stmt::Return(expr) => {
      let got = expr.map(|expr| (expr, get_expr(cx, env, fn_cx, expr)));
      match (got, fn_cx.ret_ty == Ty::Void) {
        (Some(_), true) => cx.err(stmt, ErrorKind::ReturnExprVoid),
        (None, false) => {
          cx.err(stmt, ErrorKind::ReturnNothingNonVoid(fn_cx.ret_ty))
        }
        (Some((expr, ty)), false) => {
          unify(cx, fn_cx.ret_ty, ty, expr);
        }
        (None, true) => {}
      }
      initialize(&mut fn_cx.vars, |_| true);
      true
    }
    Stmt::Block(ref stmts) => {
      let vars = fn_cx.vars.clone();
      let mut ret = BlockRet::No;
      for &stmt in stmts {
        if matches!(ret, BlockRet::Yes) {
          cx.err(stmt, ErrorKind::Unreachable);
          ret = BlockRet::YesWithUnreachable;
        }
        if get(cx, env, fn_cx, in_loop, stmt) && matches!(ret, BlockRet::No) {
          ret = BlockRet::Yes;
        }
      }
      let block_vars = std::mem::replace(&mut fn_cx.vars, vars);
      initialize(&mut fn_cx.vars, |name| block_vars[name].init);
      matches!(ret, BlockRet::Yes | BlockRet::YesWithUnreachable)
    }
    Stmt::Assert(expr) | Stmt::Error(expr) => {
      let ty = get_expr(cx, env, fn_cx, expr);
      unify(cx, Ty::Bool, ty, expr);
      false
    }
    Stmt::Break | Stmt::Continue => {
      if !in_loop {
        cx.err(stmt, ErrorKind::NotInLoop);
      }
      initialize(&mut fn_cx.vars, |_| true);
      true
    }
  }
}

enum BlockRet {
  No,
  Yes,
  YesWithUnreachable,
}

/// it used to be we would assert that if `data.init`, then `f(name)`. this was
/// a check to make sure a var doesn't go from initialized to uninitialized
/// which cannot happen in valid code.
///
/// however, we want to allow invalid code, like:
/// ```c
/// void foo() {
///   int x = 1;
///   {
///     int x;
///   }
/// }
/// ```
fn initialize<F>(vars: &mut Vars, mut f: F)
where
  F: FnMut(&Name) -> bool,
{
  for (name, data) in vars.iter_mut() {
    if !data.init && f(name) {
      data.init = true;
    }
  }
}
