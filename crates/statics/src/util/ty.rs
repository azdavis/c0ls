use core::hash::BuildHasherDefault;
use hir::Name;
use rustc_hash::FxHashMap;
use std::fmt;

#[derive(Debug)]
pub struct TyDb {
  ty_to_data: Vec<TyData>,
  data_to_ty: FxHashMap<TyData, Ty>,
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

// keep in sync with `impl Ty` and `enum TyData`
impl Default for TyDb {
  fn default() -> Self {
    let mut ret = Self {
      ty_to_data: Vec::with_capacity(Ty::LEN),
      data_to_ty: FxHashMap::with_capacity_and_hasher(
        Ty::LEN,
        BuildHasherDefault::default(),
      ),
    };
    ret.insert(TyData::None);
    ret.insert(TyData::Any);
    ret.insert(TyData::Int);
    ret.insert(TyData::Bool);
    ret.insert(TyData::String);
    ret.insert(TyData::Char);
    ret.insert(TyData::Void);
    ret.insert(TyData::Ptr(Ty::Any));
    ret.insert(TyData::Array(Ty::Any));
    assert_eq!(ret.ty_to_data.len(), Ty::LEN);
    assert_eq!(ret.data_to_ty.len(), Ty::LEN);
    ret
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ty(u32);

// keep in sync with `impl Default for TyDb` and `enum TyData`
#[allow(non_upper_case_globals)]
impl Ty {
  pub const None: Self = Self(0);
  pub const Any: Self = Self(1);
  pub const Int: Self = Self(2);
  pub const Bool: Self = Self(3);
  pub const String: Self = Self(4);
  pub const Char: Self = Self(5);
  pub const Void: Self = Self(6);
  pub const PtrAny: Self = Self(7);
  pub const ArrayAny: Self = Self(8);
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
      TyData::None => write!(f, "<none>"),
      TyData::Any => write!(f, "<any>"),
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

/// keep in sync with `impl Ty` and `impl Default for TyDb`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TyData {
  /// The 'type' of not-well-typed expressions. Permits any operation. Distinct
  /// from Any, since you cannot dereference a pointer-to-Any. Not writeable in
  /// code.
  None,
  /// Used to model the type of `NULL`. Not writeable in user code.
  Any,
  Int,
  Bool,
  String,
  Char,
  Void,
  Ptr(Ty),
  Array(Ty),
  Struct(Name),
}
