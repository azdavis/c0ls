//! Statically analyze C0 HIR.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

mod expr;
mod item;
mod root;
mod simp;
mod stmt;
mod ty;
mod util;

pub use root::get;
pub use util::error::{Error, ErrorKind, ErrorKindDisplay};
pub use util::file::{FileId, FileKind, InFile};
pub use util::id::Id;
pub use util::ty::{Ty, TyData, TyDb, TyDisplay};
pub use util::types::{
  Cx, DeclTys, Defined, Env, ExprTys, FnData, FnSig, Import, NameToTy, Param,
};
