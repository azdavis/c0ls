//! Performs static analysis on an abstract syntax tree.

#![deny(rust_2018_idioms)]

#[macro_use]
macro_rules! unwrap_or {
  ($opt:expr,$or:expr) => {
    match $opt {
      ::core::option::Option::Some(x) => x,
      ::core::option::Option::None => $or,
    }
  };
}

mod expr;
mod item;
mod root;
mod stmt;
mod ty;
mod util;

pub use root::get;
