//! inspired by rust-analyzer

use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use syntax::ast::{Cast, Syntax};
use syntax::{rowan::TextRange, SyntaxKind, SyntaxNode};

/// A 'pointer' to a SyntaxNode. Stable across re-parses of identical files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SyntaxNodePtr {
  kind: SyntaxKind,
  range: TextRange,
}

impl SyntaxNodePtr {
  fn new(node: &SyntaxNode) -> Self {
    Self {
      kind: node.kind(),
      range: node.text_range(),
    }
  }

  fn to_node(&self, mut root: SyntaxNode) -> SyntaxNode {
    assert!(root.parent().is_none());
    loop {
      if root.text_range() == self.range && root.kind() == self.kind {
        return root;
      }
      root = root
        .children()
        .find(|x| x.text_range().contains_range(self.range))
        .unwrap();
    }
  }
}

/// A 'pointer' to an AST node. Stable across re-parses of identical files.
pub struct AstPtr<N> {
  raw: SyntaxNodePtr,
  _p: PhantomData<fn() -> N>,
}

impl<N> AstPtr<N>
where
  N: Syntax,
{
  /// Returns a new `AstPtr` for the given node.
  pub fn new(node: &N) -> Self {
    Self {
      raw: SyntaxNodePtr::new(node.syntax()),
      _p: PhantomData,
    }
  }
}

impl<N> AstPtr<N>
where
  N: Cast,
{
  /// Given the root node (i.e. it has no parent) that contains the original
  /// node that the AstPtr was constructed from, return that original node.
  pub fn to_node(&self, root: SyntaxNode) -> N {
    N::cast(self.raw.to_node(root).into()).unwrap()
  }
}

impl<N> fmt::Debug for AstPtr<N> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("AstPtr")
      .field("raw", &self.raw)
      .field("_p", &self._p)
      .finish()
  }
}

impl<N> Clone for AstPtr<N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<N> Copy for AstPtr<N> {}

impl<N> PartialEq for AstPtr<N> {
  fn eq(&self, other: &AstPtr<N>) -> bool {
    self.raw == other.raw
  }
}

impl<N> Eq for AstPtr<N> {}

impl<N> Hash for AstPtr<N> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.raw.hash(state)
  }
}
