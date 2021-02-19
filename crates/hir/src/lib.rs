//! High-level intermediate representation of C0 source.
//!
//! All information about source locations, operator precedence, and basically
//! anything to do with concrete syntax is gone.
//!
//! Note that many enums have a `None` variant, to allow representing
//! partially-formed constructs. But even if a construct has a None variant, we
//! may still sometimes use Option to wrap such a construct.
//!
//! We do this to signal when it is actually allowed by the syntax for a
//! construct to be optional. If the construct is wrapped in an Option, it is
//! syntactically valid for the construct to not be present; if not, then it is
//! a syntax error if the construct is not present. But we still allow for
//! representing such syntactically invalid programs by using the None variant
//! on the construct.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use la_arena::{Arena, Idx};
use smol_str::SmolStr;
use std::borrow::Borrow;
use std::fmt;

pub use la_arena;

#[derive(Debug)]
pub struct Root {
  pub arenas: Arenas,
  pub items: Vec<ItemId>,
}

#[derive(Debug, Default)]
pub struct Arenas {
  pub item: ItemArena,
  pub expr: ExprArena,
  pub ty: TyArena,
  pub stmt: StmtArena,
  pub simp: SimpArena,
}

pub type ItemId = Idx<Item>;
pub type ItemArena = Arena<Item>;

#[derive(Debug)]
pub enum Item {
  Fn(Name, Vec<Param>, TyId, Option<StmtId>),
  Struct(Name, Vec<Field>),
  TypeDef(Name, TyId),
}

pub type TyId = Idx<Ty>;
pub type TyArena = Arena<Ty>;

#[derive(Debug)]
pub enum Ty {
  None,
  Any,
  Int,
  Bool,
  Char,
  String,
  Void,
  Ptr(TyId),
  Array(TyId),
  Struct(Name),
  Name(Name),
}

pub type ExprId = Idx<Expr>;
pub type ExprArena = Arena<Expr>;

/// `e->f` is desugared into `(*e).f`. Note that the literal expressions (int,
/// bool, char, string) do not contain the value of the literal; that's ok,
/// since we're not a compiler.
#[derive(Debug)]
pub enum Expr {
  None,
  Int,
  Bool,
  Char,
  String,
  Null,
  Name(Name),
  BinOp(ExprId, BinOp, ExprId),
  UnOp(UnOp, ExprId),
  Ternary(ExprId, ExprId, ExprId),
  Call(Name, Vec<ExprId>),
  Dot(ExprId, Name),
  Subscript(ExprId, ExprId),
  Alloc(TyId),
  AllocArray(TyId, ExprId),
}

pub type StmtId = Idx<Stmt>;
pub type StmtArena = Arena<Stmt>;

#[derive(Debug)]
pub enum Stmt {
  None,
  Simp(SimpId),
  If(ExprId, StmtId, Option<StmtId>),
  While(ExprId, StmtId),
  For(Option<SimpId>, ExprId, Option<SimpId>, StmtId),
  Return(Option<ExprId>),
  Block(Vec<StmtId>),
  Assert(ExprId),
  Error(ExprId),
  Break,
  Continue,
}

pub type SimpId = Idx<Simp>;
pub type SimpArena = Arena<Simp>;

#[derive(Debug)]
pub enum Simp {
  Assign(ExprId, AssignOp, ExprId),
  IncDec(ExprId, IncDec),
  Decl(Name, TyId, Option<ExprId>),
  Expr(ExprId),
  /// Like `foo * bar;`, which might be
  /// - a declaration of the variable `bar` of type pointer-to-`foo`, or
  /// - an expression multiplying the variables `foo` and `bar`.
  ///
  /// We cannot know which it is without knowing what typedefs are in scope.
  Ambiguous(Name, Name),
}

#[derive(Debug, Clone, Copy)]
pub enum IncDec {
  Inc,
  Dec,
}

impl fmt::Display for IncDec {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      IncDec::Inc => write!(f, "increment"),
      IncDec::Dec => write!(f, "decrement"),
    }
  }
}

#[derive(Debug)]
pub struct Param {
  pub name: Name,
  pub ty: TyId,
}

#[derive(Debug)]
pub struct Field {
  pub name: Name,
  pub ty: TyId,
}

#[derive(Debug, Clone, Copy)]
pub enum MathOp {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Shl,
  Shr,
  BitAnd,
  BitXor,
  BitOr,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
  Math(MathOp),
  Eq,
  Neq,
  Lt,
  LtEq,
  Gt,
  GtEq,
  And,
  Or,
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
  Not,
  BitNot,
  Neg,
  Deref,
}

#[derive(Debug, Clone, Copy)]
pub enum AssignOp {
  Eq,
  OpEq(MathOp),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(SmolStr);

impl Name {
  pub fn new(s: &str) -> Self {
    Self(s.into())
  }

  pub fn as_str(&self) -> &str {
    self.0.as_str()
  }
}

impl fmt::Display for Name {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl PartialEq<str> for Name {
  fn eq(&self, other: &str) -> bool {
    self.0 == other
  }
}

impl Borrow<str> for Name {
  fn borrow(&self) -> &str {
    self.0.borrow()
  }
}

impl From<&str> for Name {
  fn from(val: &str) -> Self {
    Self(val.into())
  }
}
