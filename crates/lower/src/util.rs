//! inspired by rust-analyzer

use crate::ptr::AstPtr;
use hir::la_arena::ArenaMap;
use hir::{Arenas, ExprId, ItemId, SimpId, StmtId, TyId};
use rustc_hash::FxHashMap;
use std::fmt;
use syntax::ast::{Expr, Item, Simp, Stmt, Ty};
use syntax::rowan::TextRange;

#[derive(Debug, Default)]
pub struct Ptrs {
  pub item: FxHashMap<AstPtr<Item>, ItemId>,
  pub item_back: ArenaMap<ItemId, AstPtr<Item>>,
  pub expr: FxHashMap<AstPtr<Expr>, ExprId>,
  pub expr_back: ArenaMap<ExprId, AstPtr<Expr>>,
  pub ty: FxHashMap<AstPtr<Ty>, TyId>,
  pub ty_back: ArenaMap<TyId, AstPtr<Ty>>,
  pub stmt: FxHashMap<AstPtr<Stmt>, StmtId>,
  pub stmt_back: ArenaMap<StmtId, AstPtr<Stmt>>,
  pub simp: FxHashMap<AstPtr<Simp>, SimpId>,
  pub simp_back: ArenaMap<SimpId, AstPtr<Simp>>,
}

#[derive(Debug)]
pub struct PragmaError {
  pub range: TextRange,
}

impl fmt::Display for PragmaError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "pragmas must appear before all other items")
  }
}

#[derive(Debug)]
pub struct Lowered {
  pub root: hir::Root,
  pub ptrs: Ptrs,
  pub errors: Vec<PragmaError>,
}

#[derive(Debug, Default)]
pub(crate) struct Cx {
  pub(crate) ptrs: Ptrs,
  pub(crate) arenas: Arenas,
  pub(crate) errors: Vec<PragmaError>,
}
