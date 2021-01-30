use crate::error::{ErrorKind, Thing};
use crate::name::Name;
use crate::ty::{Ty, TyData};
use crate::types::{Cx, NameToTy, VarData, VarDb};
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use syntax::{rowan::TextRange, SyntaxToken};
use unwrap_or::unwrap_or;

/// inserts value at key iff there is no existing value for key. returns whether
/// value was inserted.
pub(crate) fn insert_if_empty<K, V>(
  map: &mut FxHashMap<K, V>,
  key: K,
  value: V,
) -> bool
where
  K: std::hash::Hash + Eq,
{
  match map.entry(key) {
    Entry::Occupied(_) => false,
    Entry::Vacant(entry) => {
      entry.insert(value);
      true
    }
  }
}

pub(crate) fn unify(
  cx: &mut Cx,
  expected: Ty,
  found: Option<(TextRange, Ty)>,
) -> Ty {
  let (range, found) = unwrap_or!(found, return expected);
  unwrap_or!(unify_impl(cx, expected, found), {
    cx.error(range, ErrorKind::MismatchedTys(expected, found));
    Ty::Error
  })
}

/// produces no errors. used to implement `unify`, and exported to allow for
/// 'any' types for operators like `+`.
pub(crate) fn unify_impl(cx: &mut Cx, expected: Ty, found: Ty) -> Option<Ty> {
  // mini-optimization, also easier than writing lots of match arms
  if expected == found {
    return Some(expected);
  }
  let ret = match (cx.tys.get(expected), cx.tys.get(found)) {
    (TyData::Error, _) | (_, TyData::Error) => Ty::Error,
    (TyData::Top, _) => found,
    (_, TyData::Top) => expected,
    (&TyData::Ptr(expected), &TyData::Ptr(found)) => {
      let res = unify_impl(cx, expected, found)?;
      cx.tys.mk(TyData::Ptr(res))
    }
    (&TyData::Array(expected), &TyData::Array(found)) => {
      let res = unify_impl(cx, expected, found)?;
      cx.tys.mk(TyData::Array(res))
    }
    _ => return None,
  };
  Some(ret)
}

pub(crate) fn no_void(cx: &mut Cx, range: TextRange, ty: Ty) {
  if ty == Ty::Void {
    cx.error(range, ErrorKind::InvalidVoidTy);
  }
}

pub(crate) fn no_struct(cx: &mut Cx, range: TextRange, ty: Ty) {
  if let TyData::Struct(_) = cx.tys.get(ty) {
    cx.error(range, ErrorKind::InvalidStructTy);
  }
}

pub(crate) fn add_var(
  cx: &mut Cx,
  vars: &mut VarDb,
  type_defs: &NameToTy,
  ident: SyntaxToken,
  ty_range: TextRange,
  ty: Ty,
  defined: bool,
) {
  no_struct(cx, ty_range, ty);
  no_void(cx, ty_range, ty);
  let text = ident.text();
  let dup = type_defs.contains_key(text)
    || !insert_if_empty(vars, Name::new(text), VarData { ty, defined });
  if dup {
    cx.error(ident.text_range(), ErrorKind::Duplicate(Thing::Variable));
  }
}
