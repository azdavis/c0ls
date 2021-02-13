pub(crate) mod error;
pub(crate) mod id;
pub(crate) mod ty;
pub(crate) mod types;

use error::ErrorKind;
use id::Id;
use ty::{Ty, TyData, TyDb};
use types::{Cx, Env, Import};

pub(crate) fn unify<I: Into<Id>>(cx: &mut Cx, want: Ty, got: Ty, id: I) -> Ty {
  match unify_impl(&mut cx.tys, want, got) {
    Some(ty) => ty,
    None => {
      cx.err(id, ErrorKind::MismatchedTys(want, got));
      Ty::None
    }
  }
}

pub(crate) fn unify_impl(tys: &mut TyDb, want: Ty, got: Ty) -> Option<Ty> {
  // mini-optimization, also easier than writing lots of match arms
  if want == got {
    return Some(want);
  }
  let ret = match (tys.get(want), tys.get(got)) {
    (TyData::None, _) | (_, TyData::None) => Ty::None,
    (TyData::Any, _) => want,
    (_, TyData::Any) => got,
    (&TyData::Ptr(got), &TyData::Ptr(want)) => {
      let res = unify_impl(tys, got, want)?;
      tys.mk(TyData::Ptr(res))
    }
    (&TyData::Array(got), &TyData::Array(want)) => {
      let res = unify_impl(tys, got, want)?;
      tys.mk(TyData::Array(res))
    }
    _ => return None,
  };
  Some(ret)
}

pub(crate) fn no_void<I: Into<Id>>(cx: &mut Cx, ty: Ty, id: I) {
  if ty == Ty::Void {
    cx.err(id, ErrorKind::InvalidVoidTy);
  }
}

pub(crate) fn no_struct<I: Into<Id>>(cx: &mut Cx, ty: Ty, id: I) {
  if matches!(cx.tys.get(ty), TyData::Struct(_)) {
    cx.err(id, ErrorKind::InvalidStructTy);
  }
}

pub(crate) fn no_unsized<I: Into<Id> + Copy>(
  cx: &mut Cx,
  import: &Import,
  env: &Env,
  ty: Ty,
  id: I,
) {
  no_void(cx, ty, id);
  if let TyData::Struct(name) = cx.tys.get(ty) {
    if !import.structs.contains_key(name) && !env.structs.contains_key(name) {
      cx.err(id, ErrorKind::UndefinedStruct);
    }
  }
}
