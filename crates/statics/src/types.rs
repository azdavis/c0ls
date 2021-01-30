use crate::error::{Error, ErrorKind};
use crate::name::Name;
use crate::ty::{Ty, TyDb};
use rustc_hash::{FxHashMap, FxHashSet};
use syntax::rowan::TextRange;

#[derive(Debug, Clone, Copy)]
pub enum FileKind {
  Header,
  Source,
}

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
  /// put this on Cx since the Cx is already mutable, instead of adding it to
  /// FnData and making the ItemDb mutable throughout expr and stmt.
  pub called: FxHashSet<Name>,
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
  pub defined: Defined,
}

#[derive(Debug, Clone, Copy)]
pub enum Defined {
  /// Must not be defined.
  MustNot,
  /// Ought to be defined, but isn't yet.
  NotYet,
  /// Should be and is defined.
  Yes,
}
