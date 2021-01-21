#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

pub use event_parse;
pub use rowan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
  // trivia
  Whitespace,
  LineComment,
  BlockComment,
  /// invalid. treated as trivia. if this appears there was a corresponding
  /// error.
  Invalid,
  // identifiers
  Ident,
  // keywords
  AllocKw,
  AllocArrayKw,
  AssertKw,
  BoolKw,
  BreakKw,
  CharKw,
  ContinueKw,
  ElseKw,
  FalseKw,
  ForKw,
  IfKw,
  IntKw,
  NullKw,
  ReturnKw,
  StringKw,
  StructKw,
  TrueKw,
  TypedefKw,
  VoidKw,
  WhileKw,
  /// not listed as a keyword in the spec. an oversight?
  ErrorKw,
  /// deviation from the spec: we fail to parse unknown directives.
  UseKw,
  // punctuation
  And,
  AndAnd,
  AndEq,
  Arrow,
  Bang,
  BangEq,
  Bar,
  BarBar,
  BarEq,
  Carat,
  CaratEq,
  Colon,
  Comma,
  Dot,
  Eq,
  EqEq,
  Gt,
  GtEq,
  GtGt,
  GtGtEq,
  LCurly,
  LRound,
  LSquare,
  Lt,
  LtEq,
  LtLt,
  LtLtEq,
  Minus,
  MinusEq,
  MinusMinus,
  Percent,
  PercentEq,
  Plus,
  PlusEq,
  PlusPlus,
  Question,
  RCurly,
  RRound,
  RSquare,
  Semicolon,
  Slash,
  SlashEq,
  Star,
  StarEq,
  Tilde,
  // literals
  DecLit,
  HexLit,
  CharLit,
  StringLit,
  LibLit,
  // decls
  StructDecl,
  FnDecl,
  UseDecl,
  // defns
  StructDefn,
  FnDefn,
  TypeDefn,
  // simple stmts
  AssignStmt,
  IncStmt,
  DecStmt,
  ExprStmt,
  DeclStmt,
  DefnStmt,
  // stmts
  SimpleStmt,
  IfStmt,
  WhileStmt,
  ForStmt,
  ReturnStmt,
  BlockStmt,
  AssertStmt,
  ErrorStmt,
  // types
  PrimTy,
  PtrTy,
  ArrayTy,
  StructTy,
  IdentTy,
  // exprs
  ParenExpr,
  PrimExpr,
  IdentExpr,
  BinOpExpr,
  UnOpExpr,
  TernaryExpr,
  CallExpr,
  DotExpr,
  ArrowExpr,
  SubscriptExpr,
  AllocExpr,
  AllocArrayExpr,
  // support
  Param,
  /// root. MUST be last
  Root,
}

impl event_parse::Triviable for SyntaxKind {
  fn is_trivia(&self) -> bool {
    matches!(
      *self,
      Self::Whitespace | Self::LineComment | Self::BlockComment | Self::Invalid
    )
  }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
  fn from(kind: SyntaxKind) -> Self {
    Self(kind as u16)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum C0 {}

impl rowan::Language for C0 {
  type Kind = SyntaxKind;

  fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
    assert!(raw.0 <= SyntaxKind::Root as u16);
    unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
  }

  fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
    kind.into()
  }
}

pub type SyntaxNode = rowan::SyntaxNode<C0>;
