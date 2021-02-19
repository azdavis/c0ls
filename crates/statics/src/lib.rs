//! Statically analyze C0 HIR.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

mod expr;
mod import;
mod item;
mod root;
mod simp;
mod stmt;
mod ty;
mod util;

pub use import::add_env;
pub use root::get;
pub use util::error::{Error, ErrorKind};
pub use util::id::Id;
pub use util::ty::{Ty, TyData, TyDb};
pub use util::types::{
  Cx, Env, EnvIds, EnvWithIds, ExprTys, FileId, FnData, FnSig, Import, InFile,
  NameToTy, Param,
};
