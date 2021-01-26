pub(crate) mod error;
pub(crate) mod name;
pub(crate) mod ty;

use name::Name;
use std::collections::HashMap;
use ty::{Ty, TyData, TyDb};

/// The context. This information is always mutable as we check the various
/// constructs, and is always needed.
///
/// Other kinds of information are mutable sometimes but not other times, or not
/// always needed. For example, when checking statements, the variables in scope
/// are mutable, but for expressions, they are not. For types, we don't even
/// need to know what variables are in scope.
#[derive(Default)]
pub(crate) struct Cx {
  pub(crate) tys: TyDb,
  pub(crate) errors: error::ErrorDb,
}

pub(crate) type NameToTy = HashMap<Name, Ty>;

#[derive(Default)]
pub(crate) struct ItemDb {
  pub(crate) fns: HashMap<Name, FnData>,
  pub(crate) type_defs: NameToTy,
  pub(crate) structs: HashMap<Name, NameToTy>,
}

pub(crate) struct FnData {
  pub(crate) params: Vec<(Name, Ty)>,
  pub(crate) ret_ty: Ty,
  // TODO is the right place to put this?
  pub(crate) defined: bool,
}

pub(crate) fn unify(cx: &mut Cx, expected: Ty, found: Ty) -> Ty {
  match unify_impl(cx, expected, found) {
    Some(x) => x,
    None => todo!("issue a mismatched types error, return Ty::Error"),
  }
}

/// produces no errors. used to implement `unify`, and exported to allow for
/// 'any' types for operators like `+`.
pub(crate) fn unify_impl(cx: &mut Cx, expected: Ty, found: Ty) -> Option<Ty> {
  // mini-optimization, also easier than writing lots of match arms
  if expected == found {
    return Some(expected);
  }
  match (cx.tys.get(expected), cx.tys.get(found)) {
    (TyData::Error, _) | (_, TyData::Error) => Some(Ty::Error),
    (TyData::Top, _) => Some(found),
    (_, TyData::Top) => Some(expected),
    (&TyData::Ptr(expected), &TyData::Ptr(found)) => {
      let res = unify(cx, expected, found);
      Some(cx.tys.mk(TyData::Ptr(res)))
    }
    (&TyData::Array(expected), &TyData::Array(found)) => {
      let res = unify(cx, expected, found);
      Some(cx.tys.mk(TyData::Array(res)))
    }
    _ => None,
  }
}