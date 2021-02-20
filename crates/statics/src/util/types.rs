use crate::util::error::{Error, ErrorKind};
use crate::util::id::Id;
use crate::util::ty::{Ty, TyDb};
use hir::{la_arena::ArenaMap, Arenas, ExprId, ItemId, Name, TyId};
use rustc_hash::FxHashMap;
use std::fmt;
use uri_db::UriId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileId {
  /// Not a real file the user wrote, just the standard library.
  StdLib,
  /// A real file that exists at a URI.
  Uri(UriId),
}

impl FileId {
  pub fn wrap<T>(self, val: T) -> InFile<T> {
    InFile { file: self, val }
  }

  pub fn uri(&self) -> Option<UriId> {
    match *self {
      Self::StdLib => None,
      Self::Uri(uri) => Some(uri),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InFile<T> {
  file: FileId,
  val: T,
}

impl<T> InFile<T> {
  pub fn file(&self) -> FileId {
    self.file
  }

  pub fn val(&self) -> &T {
    &self.val
  }
}

#[derive(Debug, Clone)]
pub struct FnSig {
  pub params: Vec<Param>,
  pub ret_ty: Ty,
  pub is_defined: bool,
  pub should_define: bool,
}

impl FnSig {
  pub fn display<'a>(
    &'a self,
    name: &'a Name,
    tys: &'a TyDb,
  ) -> impl fmt::Display + 'a {
    FnSigDisplay {
      this: self,
      name,
      tys,
    }
  }
}

struct FnSigDisplay<'a> {
  this: &'a FnSig,
  name: &'a Name,
  tys: &'a TyDb,
}

impl fmt::Display for FnSigDisplay<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} {}(", self.this.ret_ty.display(self.tys), self.name)?;
    let mut params = self.this.params.iter();
    if let Some(param) = params.next() {
      write!(f, "{}", param.display(self.tys))?;
    }
    for param in params {
      write!(f, ", {}", param.display(self.tys))?;
    }
    write!(f, ")")
  }
}

pub type NameToTy = FxHashMap<Name, Ty>;

#[derive(Debug, Clone)]
pub struct Param {
  /// only used for informational messages
  pub name: Name,
  pub ty: Ty,
}

impl Param {
  fn display<'a>(&'a self, tys: &'a TyDb) -> impl fmt::Display + 'a {
    ParamDisplay { this: self, tys }
  }
}

struct ParamDisplay<'a> {
  this: &'a Param,
  tys: &'a TyDb,
}

impl fmt::Display for ParamDisplay<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} {}", self.this.ty.display(self.tys), self.this.name)
  }
}

#[derive(Debug, Default)]
pub struct Import {
  pub fns: FxHashMap<Name, InFile<FnSig>>,
  pub structs: FxHashMap<Name, InFile<NameToTy>>,
  pub type_defs: FxHashMap<Name, InFile<Ty>>,
}

impl Import {
  pub fn with_main() -> Self {
    let mut ret = Self::default();
    ret.fns.insert(
      "main".into(),
      // this is the only thing 'from' the std lib that can be defined.
      FileId::StdLib.wrap(FnSig {
        params: vec![],
        ret_ty: Ty::Int,
        is_defined: false,
        should_define: true,
      }),
    );
    ret
  }
}

pub type ExprTys = ArenaMap<ExprId, Ty>;

/// this is useful when resolving typedefs to display a given type, but not
/// strictly necessary since we could just traverse to know what typedefs are in
/// scope
pub type TyTys = ArenaMap<TyId, Ty>;

#[derive(Debug)]
pub struct FnData {
  pub sig: FnSig,
}

#[derive(Debug, Default)]
pub struct Env {
  pub fns: FxHashMap<Name, FnData>,
  pub structs: FxHashMap<Name, NameToTy>,
  pub type_defs: FxHashMap<Name, Ty>,
  pub expr_tys: ExprTys,
  pub ty_tys: TyTys,
}

#[derive(Debug, Default)]
pub struct EnvIds {
  pub fns: FxHashMap<Name, ItemId>,
  pub structs: FxHashMap<Name, ItemId>,
  pub type_defs: FxHashMap<Name, ItemId>,
}

/// TODO not great. this and [`EnvIds`] are only used for imports.
#[derive(Debug, Default)]
pub struct EnvWithIds {
  pub env: Env,
  pub ids: EnvIds,
}

#[derive(Debug, Default)]
pub struct Cx {
  pub tys: TyDb,
  pub errors: Vec<Error>,
}

pub(crate) type Vars = FxHashMap<Name, VarData>;

#[derive(Debug, Clone)]
pub(crate) struct VarData {
  pub(crate) ty: Ty,
  pub(crate) init: bool,
}

impl Cx {
  pub(crate) fn err<I: Into<Id>>(&mut self, id: I, kind: ErrorKind) {
    self.errors.push(Error {
      id: id.into(),
      kind,
    });
  }
}

pub(crate) struct FnCx<'a> {
  pub import: &'a Import,
  pub arenas: &'a Arenas,
  pub vars: Vars,
  pub ret_ty: Ty,
}
