use crate::expr::{get as get_expr, get_name as get_name_expr};
use crate::ty::get as get_ty;
use crate::util::error::ErrorKind;
use crate::util::ty::{Ty, TyData};
use crate::util::types::{Cx, Env, FnCx, Import, InFile, VarData};
use crate::util::{no_struct, no_void, unify};
use hir::{Arenas, AssignOp, Expr, ExprId, Name, Simp, SimpId, UnOp};

pub(crate) enum VarInfo<'a> {
  None,
  /// must have already been decl
  Defn(&'a Name),
  Decl,
}

pub(crate) fn get<'a>(
  cx: &mut Cx,
  env: &mut Env,
  fn_cx: &mut FnCx<'a>,
  simp: SimpId,
) -> VarInfo<'a> {
  let mut ret = VarInfo::None;
  match fn_cx.arenas.simp[simp] {
    Simp::Assign(lhs, op, rhs) => {
      let rhs_ty = get_expr(cx, env, fn_cx, rhs);
      let want_lhs_ty = match op {
        AssignOp::Eq => rhs_ty,
        AssignOp::OpEq(_) => {
          unify(cx, Ty::Int, rhs_ty, rhs);
          Ty::Int
        }
      };
      match get_lv(fn_cx.import, fn_cx.arenas, lhs) {
        Some(Lv::Name(name)) => {
          if matches!(op, AssignOp::Eq) {
            if let Some(data) = fn_cx.vars.get_mut(name) {
              data.init = true;
              ret = VarInfo::Defn(name);
            }
          }
        }
        Some(Lv::Other) => {}
        None => cx.err(lhs, ErrorKind::CannotAssign),
      }
      let lhs_ty = get_expr(cx, env, fn_cx, lhs);
      unify(cx, want_lhs_ty, lhs_ty, lhs);
    }
    Simp::IncDec(expr, inc_dec) => {
      if get_lv(fn_cx.import, fn_cx.arenas, expr).is_none() {
        cx.err(expr, ErrorKind::CannotIncDec(inc_dec));
      }
      let ty = get_expr(cx, env, fn_cx, expr);
      unify(cx, Ty::Int, ty, expr);
    }
    Simp::Decl(ref name, ty, expr) => {
      let got_ty = get_ty(fn_cx.import, fn_cx.arenas, cx, env, ty);
      let init = match expr {
        None => false,
        Some(expr) => {
          let expr_ty = get_expr(cx, env, fn_cx, expr);
          unify(cx, got_ty, expr_ty, expr);
          true
        }
      };
      no_void(cx, got_ty, ty);
      no_struct(cx, got_ty, ty);
      let data = VarData { ty: got_ty, init };
      let dup = fn_cx.vars.insert(name.clone(), data).is_some()
        || env.type_defs.contains_key(name)
        || fn_cx.import.type_defs.contains_key(name);
      if dup {
        cx.err(simp, ErrorKind::Duplicate(name.clone()));
      }
      ret = VarInfo::Decl;
    }
    Simp::Expr(expr) => {
      let ty = get_expr(cx, env, fn_cx, expr);
      no_struct(cx, ty, expr);
    }
    // hacky. using `simp` as the ID for the error is not great.
    Simp::Ambiguous(ref lhs, ref rhs) => {
      let ty = env
        .type_defs
        .get(lhs)
        .or_else(|| fn_cx.import.type_defs.get(lhs).map(InFile::val));
      match ty {
        // multiplication.
        None => {
          let lhs_ty = get_name_expr(cx, &fn_cx.vars, simp, lhs);
          let rhs_ty = get_name_expr(cx, &fn_cx.vars, simp, rhs);
          unify(cx, Ty::Int, lhs_ty, simp);
          unify(cx, Ty::Int, rhs_ty, simp);
        }
        // declaration. largely duplicated from Simp::Decl.
        Some(&ty) => {
          let ty = cx.tys.mk(TyData::Ptr(ty));
          no_void(cx, ty, simp);
          no_struct(cx, ty, simp);
          let data = VarData { ty, init: false };
          let dup = fn_cx.vars.insert(rhs.clone(), data).is_some()
            || env.type_defs.contains_key(rhs)
            || fn_cx.import.type_defs.contains_key(rhs);
          if dup {
            cx.err(simp, ErrorKind::Duplicate(rhs.clone()));
          }
          ret = VarInfo::Decl;
        }
      }
    }
  }
  ret
}

enum Lv<'a> {
  Name(&'a Name),
  Other,
}

fn get_lv<'a>(
  import: &Import,
  arenas: &'a Arenas,
  expr: ExprId,
) -> Option<Lv<'a>> {
  match arenas.expr[expr] {
    Expr::Name(ref name) => Some(Lv::Name(name)),
    Expr::UnOp(op, expr) => match op {
      UnOp::Not | UnOp::BitNot | UnOp::Neg => None,
      UnOp::Deref => get_lv(import, arenas, expr).map(|_| Lv::Other),
    },
    Expr::FieldGet(expr, _) | Expr::Subscript(expr, _) => {
      get_lv(import, arenas, expr).map(|_| Lv::Other)
    }
    Expr::None
    | Expr::Int
    | Expr::Bool
    | Expr::Char
    | Expr::String
    | Expr::Null
    | Expr::BinOp(_, _, _)
    | Expr::Ternary(_, _, _)
    | Expr::Call(_, _)
    | Expr::Alloc(_)
    | Expr::AllocArray(_, _) => None,
  }
}
