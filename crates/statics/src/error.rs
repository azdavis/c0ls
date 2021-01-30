use crate::ty::Ty;
use syntax::rowan::TextRange;

#[derive(Debug)]
pub struct Error {
  pub range: TextRange,
  pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
  Duplicate(Thing),
  Undefined(Thing),
  MismatchedNumParams(usize, usize),
  DerefNull,
  DerefNonPtr(Ty),
  MismatchedNumArgs(usize, usize),
  SubscriptNonArray(Ty),
  FieldGetNonStruct(Ty),
  MismatchedTys(Ty, Ty),
  CallNonFn(Ty),
  MismatchedTysAny(&'static [Ty], Ty),
  InvalidVoidTy,
  InvalidStructTy,
  InvalidNoReturn,
  #[cfg(feature = "unreachable")]
  Unreachable,
  InvalidStepDecl,
  ReturnExprVoid,
  NoReturnExprNotVoid,
  /// None means it's just regular assignment (=, +=, etc).
  InvalidAssign(Option<IncDec>),
  UninitializedVar,
  BreakOutsideLoop,
  ContinueOutsideLoop,
  DerefIncDec(IncDec),
  DefnOfHeaderFn,
  PragmaNotFirst,
}

#[derive(Debug, Clone, Copy)]
pub enum IncDec {
  Inc,
  Dec,
}

impl IncDec {
  pub fn display(&self) -> &'static str {
    match *self {
      IncDec::Inc => "increment",
      IncDec::Dec => "decrement",
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum Thing {
  Field,
  Struct,
  Variable,
  Function,
  Typedef,
}

impl Thing {
  pub fn display(&self) -> &'static str {
    match *self {
      Thing::Field => "field",
      Thing::Struct => "struct",
      Thing::Variable => "variable",
      Thing::Function => "function",
      Thing::Typedef => "typedef",
    }
  }
}
