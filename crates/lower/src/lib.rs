//! Lowers AST into HIR.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

mod expr;
mod item;
mod root;
mod simp;
mod stmt;
mod ty;
mod util;

pub use root::get;
pub use util::{Lowered, PragmaError, Ptrs};
