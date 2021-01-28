use crate::name::Name;
use core::hash::BuildHasherDefault;
use rustc_hash::FxHashMap;

/// A type store. Do not pass [`Ty`]s returned by one `TyDb` to another `TyDb`.
pub struct TyDb {
  ty_to_data: Vec<TyData>,
  data_to_ty: FxHashMap<TyData, Ty>,
}

impl TyDb {
  pub(crate) fn mk(&mut self, data: TyData) -> Ty {
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
      data_to_ty: FxHashMap::with_capacity_and_hasher(
        Ty::LEN,
        BuildHasherDefault::default(),
      ),
    };
    ret.insert(TyData::Error);
    ret.insert(TyData::Top);
    ret.insert(TyData::Int);
    ret.insert(TyData::Bool);
    ret.insert(TyData::String);
    ret.insert(TyData::Char);
    ret.insert(TyData::Void);
    ret.insert(TyData::Ptr(Ty::Top));
    ret.insert(TyData::Array(Ty::Top));
    assert_eq!(ret.ty_to_data.len(), Ty::LEN);
    assert_eq!(ret.data_to_ty.len(), Ty::LEN);
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
  pub(crate) const Error: Self = Self(0);
  pub(crate) const Top: Self = Self(1);
  pub(crate) const Int: Self = Self(2);
  pub(crate) const Bool: Self = Self(3);
  pub(crate) const String: Self = Self(4);
  pub(crate) const Char: Self = Self(5);
  pub(crate) const Void: Self = Self(6);
  pub(crate) const PtrTop: Self = Self(7);
  pub(crate) const ArrayTop: Self = Self(8);
  const LEN: usize = 9;
}

/// Data about a type. Give this to a [`TyDb`] to get back a corresponding
/// [`Ty`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TyData {
  /// The 'type' of not-well-typed expressions. Permits any operation. Distinct
  /// from Top, since you cannot dereference a pointer-to-Top.
  Error,
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
