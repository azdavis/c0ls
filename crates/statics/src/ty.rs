use crate::name::Name;
use core::hash::BuildHasherDefault;
use rustc_hash::FxHashMap;
use std::fmt;

/// A type store. Do not pass [`Ty`]s returned by one `TyDb` to another `TyDb`.
#[derive(Debug)]
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

/// A type, issued by a [`TyDb`].
///
/// Do not mix `Ty`s issued by different `TyDb`s. However, the associated
/// constants on `impl Ty` will always be the same across different `TyDb`s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ty(u32);

/// keep in sync with `impl Default for TyDb`
#[allow(non_upper_case_globals)]
impl Ty {
  pub const Error: Self = Self(0);
  pub const Top: Self = Self(1);
  pub const Int: Self = Self(2);
  pub const Bool: Self = Self(3);
  pub const String: Self = Self(4);
  pub const Char: Self = Self(5);
  pub const Void: Self = Self(6);
  pub const PtrTop: Self = Self(7);
  pub const ArrayTop: Self = Self(8);
  const LEN: usize = 9;

  pub fn display(self, tys: &TyDb) -> TyDisplay<'_> {
    TyDisplay { ty: self, tys }
  }
}

#[derive(Debug)]
pub struct TyDisplay<'a> {
  ty: Ty,
  tys: &'a TyDb,
}

impl fmt::Display for TyDisplay<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self.tys.get(self.ty) {
      TyData::Error => write!(f, "<error>"),
      TyData::Top => write!(f, "<any>"),
      TyData::Int => write!(f, "int"),
      TyData::Bool => write!(f, "bool"),
      TyData::String => write!(f, "string"),
      TyData::Char => write!(f, "char"),
      TyData::Void => write!(f, "void"),
      TyData::Ptr(t) => write!(f, "{}*", t.display(self.tys)),
      TyData::Array(t) => write!(f, "{}[]", t.display(self.tys)),
      TyData::Struct(ref name) => write!(f, "struct {}", name),
    }
  }
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
