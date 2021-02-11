use crate::util::error::{Error, ErrorKind};
use crate::util::id::Id;
use crate::util::ty::{Ty, TyDb};
use hir::{la_arena::ArenaMap, Arenas, ExprId, Name, SimpId};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Clone, Copy)]
pub enum Defined {
  /// Must not be defined.
  MustNot,
  /// Ought to be defined, but isn't yet.
  NotYet,
  /// Should be and is defined.
  Yes,
}

#[derive(Debug, Clone)]
pub struct FnSig {
  pub params: Vec<Param>,
  pub ret_ty: Ty,
  pub defined: Defined,
}

pub type NameToTy = FxHashMap<Name, Ty>;

#[derive(Debug, Clone)]
pub struct Param {
  /// only used for informational messages
  pub name: Name,
  pub ty: Ty,
}

#[derive(Debug)]
pub struct Import {
  pub fns: FxHashMap<Name, FnSig>,
  pub structs: FxHashMap<Name, NameToTy>,
  pub type_defs: FxHashMap<Name, Ty>,
}

impl Default for Import {
  fn default() -> Self {
    let mut fns = FxHashMap::default();
    fns.insert(
      "main".into(),
      FnSig {
        params: vec![],
        ret_ty: Ty::Int,
        defined: Defined::NotYet,
      },
    );
    Self {
      fns,
      structs: FxHashMap::default(),
      type_defs: FxHashMap::default(),
    }
  }
}

pub type DeclTys = ArenaMap<SimpId, Ty>;
pub type ExprTys = ArenaMap<ExprId, Ty>;

#[derive(Debug)]
pub struct FnData {
  pub sig: FnSig,
  // pub decl_tys: DeclTys,
  // pub expr_tys: ExprTys,
}

#[derive(Debug, Default)]
pub struct Env {
  pub fns: FxHashMap<Name, FnData>,
  pub structs: FxHashMap<Name, NameToTy>,
  pub type_defs: FxHashMap<Name, Ty>,
  pub decl_tys: DeclTys,
  pub expr_tys: ExprTys,
  pub called_fns: FxHashSet<Name>,
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
  // pub decl_tys: DeclTys,
  // pub expr_tys: ExprTys,
}