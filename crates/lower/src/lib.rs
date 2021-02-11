//! Lowers AST into HIR.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

mod expr;
mod item;
mod ptr;
mod root;
mod simp;
mod stmt;
mod ty;
mod util;

pub use ptr::AstPtr;
pub use root::get;
pub use util::{Lowered, PragmaError, Ptrs, SyntheticSyntax};
