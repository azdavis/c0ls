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

pub fn unify(tys: &mut TyDb, ty1: Ty, ty2: Ty) -> Option<Ty> {
  // mini-optimization, also easier than writing lots of match arms
  if ty1 == ty2 {
    return Some(ty1);
  }
  match (tys.get(ty1), tys.get(ty2)) {
    (TyData::Top, _) => Some(ty2),
    (_, TyData::Top) => Some(ty1),
    (&TyData::Ptr(ty1), &TyData::Ptr(ty2)) => {
      let res = unify(tys, ty1, ty2)?;
      Some(tys.mk(TyData::Ptr(res)))
    }
    (&TyData::Array(ty1), &TyData::Array(ty2)) => {
      let res = unify(tys, ty1, ty2)?;
      Some(tys.mk(TyData::Array(res)))
    }
    _ => None,
  }
}
