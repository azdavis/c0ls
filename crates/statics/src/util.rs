use crate::error::{Error, ErrorKind, Thing};
use crate::name::Name;
use crate::ty::{Ty, TyData, TyDb};
use rustc_hash::FxHashMap;
use std::collections::hash_map::Entry;
use syntax::{rowan::TextRange, SyntaxToken};
use unwrap_or::unwrap_or;

/// The context. This information is always mutable as we check the various
/// constructs, and is always needed.
///
/// Other kinds of information are mutable sometimes but not other times, or not
/// always needed. For example, when checking statements, the variables in scope
/// are mutable, but for expressions, they are not. For types, we don't even
/// need to know what variables are in scope.
#[derive(Debug, Default)]
pub struct Cx {
  pub tys: TyDb,
  pub errors: Vec<Error>,
}

impl Cx {
  pub(crate) fn error(&mut self, range: TextRange, kind: ErrorKind) {
    self.errors.push(Error { range, kind });
  }
}

pub(crate) type NameToTy = FxHashMap<Name, Ty>;

pub(crate) type VarDb = FxHashMap<Name, VarData>;

#[derive(Debug, Clone, Copy)]
pub struct VarData {
  pub ty: Ty,
  pub defined: bool,
}

#[derive(Debug, Default)]
pub struct ItemDb {
  pub fns: FxHashMap<Name, FnData>,
  pub type_defs: NameToTy,
  pub structs: FxHashMap<Name, NameToTy>,
}

#[derive(Debug)]
pub struct FnData {
  pub params: Vec<(Name, Ty)>,
  pub ret_ty: Ty,
  pub defined: bool,
}

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
    cx.error(range, ErrorKind::MismatchedTypes(expected, found));
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
    cx.error(range, ErrorKind::InvalidVoid);
  }
}

pub(crate) fn no_struct(cx: &mut Cx, range: TextRange, ty: Ty) {
  if let TyData::Struct(_) = cx.tys.get(ty) {
    cx.error(range, ErrorKind::InvalidStruct);
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
