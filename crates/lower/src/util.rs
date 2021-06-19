//! inspired by rust-analyzer

use hir::la_arena::ArenaMap;
use hir::{Arenas, ExprId, ItemId, SimpId, StmtId, TyId};
use rustc_hash::FxHashMap;
use std::fmt;
use syntax::ast::{Expr, Item, Simp, Stmt, Ty};
use syntax::rowan::TextRange;
use syntax::AstPtr;

/// Pointers between the AST and the HIR.
///
/// For example, we may wish to pass only HIR to some function (e.g. when
/// checking the statics for some construct), but then if the construct has an
/// error, we need to know where it was in the source so we can show the error
/// in the right place.
///
/// Or, we may have a location in the source, and want to translate that into a
/// HIR construct.
#[derive(Debug, Default)]
#[allow(missing_docs)]
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

/// An error for when a pragma appeared after some non-pragma item.
#[derive(Debug)]
pub struct PragmaError {
  /// The range of the pragma.
  pub range: TextRange,
}

impl fmt::Display for PragmaError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "pragmas must appear before all other items")
  }
}

/// The result of lowering.
#[derive(Debug)]
pub struct Lowered {
  /// The HIR root.
  pub root: hir::Root,
  /// The pointers between the HIR root and the AST root that it was lowered
  /// from.
  pub ptrs: Ptrs,
  /// The errors encountered when lowering.
  pub errors: Vec<PragmaError>,
}

#[derive(Debug, Default)]
pub(crate) struct Cx {
  pub(crate) ptrs: Ptrs,
  pub(crate) arenas: Arenas,
  pub(crate) errors: Vec<PragmaError>,
}
