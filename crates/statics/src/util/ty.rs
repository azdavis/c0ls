use crate::util::name::Name;
use std::collections::HashMap;

/// A type store. Do not pass [`Ty`]s returned by one `TyDb` to another `TyDb`.
pub struct TyDb {
  ty_to_data: Vec<TyData>,
  data_to_ty: HashMap<TyData, Ty>,
}

impl TyDb {
  pub fn mk(&mut self, data: TyData) -> Ty {
    if let Some(&ty) = self.data_to_ty.get(&data) {
      return ty;
    }
    self.insert(data)
  }

  fn insert(&mut self, data: TyData) -> Ty {
    let ret = Ty(self.ty_to_data.len() as u32);
    assert!(self.data_to_ty.insert(data.clone(), ret).is_none());
    self.ty_to_data.push(data);
    ret
  }

  pub fn get(&self, ty: Ty) -> &TyData {
    self.ty_to_data.get(ty.0 as usize).expect("no data for ty")
  }
}

/// keep in sync with `impl Ty`
impl Default for TyDb {
  fn default() -> Self {
    let mut ret = Self {
      ty_to_data: Vec::with_capacity(Ty::LEN),
      data_to_ty: HashMap::with_capacity(Ty::LEN),
    };
    ret.insert(TyData::Top);
    ret.insert(TyData::Int);
    ret.insert(TyData::Bool);
    ret.insert(TyData::String);
    ret.insert(TyData::Char);
    ret.insert(TyData::Void);
    ret.insert(TyData::Ptr(Ty::Top));
    ret.insert(TyData::Array(Ty::Top));
    ret
  }
}

/// A type, issued by a [`TyDb`]. Do not mix [`Ty`]s issued by different
/// [`TyDb`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ty(u32);

/// keep in sync with `impl Default for TyDb`
#[allow(non_upper_case_globals)]
impl Ty {
  pub const Top: Self = Self(0);
  pub const Int: Self = Self(1);
  pub const Bool: Self = Self(2);
  pub const String: Self = Self(3);
  pub const Char: Self = Self(4);
  pub const Void: Self = Self(5);
  pub const PtrTop: Self = Self(6);
  pub const ArrayTop: Self = Self(7);
  const LEN: usize = 7;
}

/// Data about a type. Give this to a [`TyDb`] to get back a corresponding
/// [`Ty`]. Note the lack of `void`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TyData {
  /// 'Any' type. Used to model the type of `NULL`. Not writeable in user code.
  Top,
  Int,
  Bool,
  String,
  Char,
  Void,
  Ptr(Ty),
  Array(Ty),
  Struct(Name),
}
