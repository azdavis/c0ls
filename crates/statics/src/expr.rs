use crate::ty::get as get_ty;
use crate::util::error::ErrorKind;
use crate::util::id::Id;
use crate::util::ty::{Ty, TyData};
use crate::util::types::{Cx, Env, FnCx, Vars};
use crate::util::{no_struct, no_unsized, no_void, unify, unify_impl};
use hir::{BinOp, Expr, ExprId, Name, UnOp};

pub(crate) fn get(
  cx: &mut Cx,
  env: &mut Env,
  fn_cx: &mut FnCx<'_>,
  expr: ExprId,
) -> Ty {
  let ret = match fn_cx.arenas.expr[expr] {
    Expr::None => Ty::None,
    Expr::Int => Ty::Int,
    Expr::Bool => Ty::Bool,
    Expr::Char => Ty::Char,
    Expr::String => Ty::String,
    Expr::Null => Ty::PtrAny,
    Expr::Name(ref name) => get_name(cx, &fn_cx.vars, expr, name),
    Expr::BinOp(lhs, op, rhs) => {
      let lhs_ty = get(cx, env, fn_cx, lhs);
      let rhs_ty = get(cx, env, fn_cx, rhs);
      let (param_tys, ret_ty) = bin_op_ty(op);
      let param_ty = param_tys
        .iter()
        .find_map(|&p| unify_impl(&mut cx.tys, p, lhs_ty));
      match param_ty {
        None => cx.err(lhs, ErrorKind::MismatchedTysAny(param_tys, lhs_ty)),
        Some(ty) => {
          unify(cx, ty, rhs_ty, rhs);
        }
      }
      ret_ty
    }
    Expr::UnOp(op, inner) => {
      let ty = get(cx, env, fn_cx, inner);
      match op {
        UnOp::Not => {
          unify(cx, Ty::Bool, ty, inner);
          Ty::Bool
        }
        UnOp::BitNot | UnOp::Neg => {
          unify(cx, Ty::Int, ty, inner);
          Ty::Int
        }
        UnOp::Deref => match *cx.tys.get(ty) {
          TyData::None => Ty::None,
          TyData::Ptr(ty) => {
            if ty == Ty::Any {
              cx.err(expr, ErrorKind::DerefNull);
              Ty::None
            } else {
              ty
            }
          }
          _ => {
            cx.err(expr, ErrorKind::DerefNonPtrTy(ty));
            Ty::None
          }
        },
      }
    }
    Expr::Ternary(cond, yes, no) => {
      let cond_ty = get(cx, env, fn_cx, cond);
      let yes_ty = get(cx, env, fn_cx, yes);
      let no_ty = get(cx, env, fn_cx, no);
      unify(cx, Ty::Bool, cond_ty, cond);
      let ret = unify(cx, yes_ty, no_ty, no);
      no_void(cx, ret, expr);
      no_struct(cx, ret, expr);
      ret
    }
    Expr::Call(ref name, ref args) => {
      env.called_fns.insert(name.clone());
      // do this here non-lazily since we want all type errors from trying to
      // get the types of the arguments. could probably wrangle doing this
      // non-lazily without allocating but it would require more finagling to
      // keep the borrow checker happy.
      let got: Vec<_> = args
        .iter()
        .map(|&expr| (get(cx, env, fn_cx, expr), expr))
        .collect();
      if let Some(data) = fn_cx.vars.get(name) {
        cx.err(expr, ErrorKind::CallNonFnTy(data.ty));
      }
      let sig = env
        .fns
        .get(name)
        .map(|x| &x.sig)
        .or_else(|| fn_cx.import.fns.get(name));
      match sig {
        Some(sig) => {
          let want_len = sig.params.len();
          if want_len != got.len() {
            cx.err(expr, ErrorKind::MismatchedNumArgs(want_len, got.len()));
          }
          for (want, (got, expr)) in sig.params.iter().zip(got) {
            unify(cx, want.ty, got, expr);
          }
          sig.ret_ty
        }
        None => {
          cx.err(expr, ErrorKind::UndefinedFn);
          Ty::None
        }
      }
    }
    Expr::Dot(inner, ref field) => {
      let ty = get(cx, env, fn_cx, inner);
      let mut ret = Ty::None;
      match cx.tys.get(ty) {
        TyData::None => {}
        TyData::Struct(name) => {
          let sig = env
            .structs
            .get(name)
            .or_else(|| fn_cx.import.structs.get(name));
          match sig {
            None => cx.err(inner, ErrorKind::UndefinedStruct),
            Some(sig) => match sig.get(field) {
              None => cx.err(expr, ErrorKind::UndefinedField),
              Some(&ty) => ret = ty,
            },
          }
        }
        _ => cx.err(expr, ErrorKind::FieldGetNonStructTy(ty)),
      }
      ret
    }
    Expr::Subscript(array, idx) => {
      let array_ty = get(cx, env, fn_cx, array);
      let idx_ty = get(cx, env, fn_cx, idx);
      unify(cx, Ty::Int, idx_ty, idx);
      match *cx.tys.get(array_ty) {
        TyData::None => Ty::None,
        TyData::Array(ty) => ty,
        _ => {
          cx.err(expr, ErrorKind::SubscriptNonArrayTy(array_ty));
          Ty::None
        }
      }
    }
    Expr::Alloc(ty) => {
      let got_ty = get_ty(fn_cx.import, fn_cx.arenas, cx, env, ty);
      no_unsized(cx, fn_cx.import, env, got_ty, ty);
      cx.tys.mk(TyData::Ptr(got_ty))
    }
    Expr::AllocArray(ty, len) => {
      let got_ty = get_ty(fn_cx.import, fn_cx.arenas, cx, env, ty);
      no_unsized(cx, fn_cx.import, env, got_ty, ty);
      let len_ty = get(cx, env, fn_cx, len);
      unify(cx, Ty::Int, len_ty, len);
      cx.tys.mk(TyData::Array(got_ty))
    }
  };
  env.expr_tys.insert(expr, ret);
  ret
}

/// only pub(crate) as its own function because of Simp::Ambiguous
pub(crate) fn get_name<I: Into<Id>>(
  cx: &mut Cx,
  vars: &Vars,
  id: I,
  name: &Name,
) -> Ty {
  match vars.get(name) {
    None => {
      cx.err(id, ErrorKind::UndefinedVar);
      Ty::None
    }
    Some(var_data) => {
      if !var_data.init {
        cx.err(id, ErrorKind::UninitializedVar);
      }
      var_data.ty
    }
  }
}

fn bin_op_ty(op: BinOp) -> (&'static [Ty], Ty) {
  let param_tys: &'static [Ty];
  let ret_ty;
  match op {
    BinOp::Math(_) => {
      param_tys = &[Ty::Int];
      ret_ty = Ty::Int;
    }
    BinOp::Eq | BinOp::Neq => {
      param_tys = &[Ty::Int, Ty::Bool, Ty::Char, Ty::PtrAny, Ty::ArrayAny];
      ret_ty = Ty::Bool;
    }
    BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
      param_tys = &[Ty::Int, Ty::Char];
      ret_ty = Ty::Bool;
    }
    BinOp::And | BinOp::Or => {
      param_tys = &[Ty::Bool];
      ret_ty = Ty::Bool;
    }
  }
  (param_tys, ret_ty)
}
