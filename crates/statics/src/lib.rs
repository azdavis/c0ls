//! Performs static analysis on an abstract syntax tree.

#![deny(rust_2018_idioms)]

mod expr;
mod item;
mod root;
mod stmt;
mod ty;
mod util;

pub use root::get;
