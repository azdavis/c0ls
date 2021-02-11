use hir::{ExprId, ItemId, SimpId, StmtId, TyId};

#[derive(Debug, Clone, Copy)]
pub enum Id {
  Expr(ExprId),
  Ty(TyId),
  Stmt(StmtId),
  Simp(SimpId),
  Item(ItemId),
}

impl From<ExprId> for Id {
  fn from(val: ExprId) -> Self {
    Self::Expr(val)
  }
}

impl From<TyId> for Id {
  fn from(val: TyId) -> Self {
    Self::Ty(val)
  }
}

impl From<StmtId> for Id {
  fn from(val: StmtId) -> Self {
    Self::Stmt(val)
  }
}

impl From<SimpId> for Id {
  fn from(val: SimpId) -> Self {
    Self::Simp(val)
  }
}

impl From<ItemId> for Id {
  fn from(val: ItemId) -> Self {
    Self::Item(val)
  }
}
