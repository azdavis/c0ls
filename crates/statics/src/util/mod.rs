pub mod name;
pub mod ty;

use name::Name;
use std::collections::HashMap;
use ty::{Ty, TyData, TyDb};

pub type NameToTy = HashMap<Name, Ty>;

#[derive(Default)]
pub struct ItemDb {
  pub fns: HashMap<Name, FnData>,
  pub type_defs: NameToTy,
  pub structs: HashMap<Name, NameToTy>,
}

pub struct FnData {
  pub params: Vec<(Name, Ty)>,
  pub ret_ty: Ty,
  // TODO is the right place to put this?
  pub defined: bool,
}

pub fn unify(tys: &mut TyDb, expected: Ty, found: Ty) -> Option<Ty> {
  // mini-optimization, also easier than writing lots of match arms
  if expected == found {
    return Some(expected);
  }
  match (tys.get(expected), tys.get(found)) {
    (TyData::Top, _) => Some(found),
    (_, TyData::Top) => Some(expected),
    (&TyData::Ptr(expected), &TyData::Ptr(found)) => {
      let res = unify(tys, expected, found)?;
      Some(tys.mk(TyData::Ptr(res)))
    }
    (&TyData::Array(expected), &TyData::Array(found)) => {
      let res = unify(tys, expected, found)?;
      Some(tys.mk(TyData::Array(res)))
    }
    _ => None,
  }
}
