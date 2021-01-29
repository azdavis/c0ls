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
  MismatchedTypes(Ty, Ty),
  ShadowedFunction,
  MismatchedTypesAny(&'static [Ty], Ty),
  InvalidVoid,
  InvalidStruct,
  InvalidNoReturn,
  Unreachable,
  InvalidStepDecl,
  ReturnExprVoid,
  NoReturnExprNotVoid,
  /// None means it's just regular assignment (=, +=, etc).
  CannotAssign(Option<IncDec>),
  UninitializedVar,
  BreakOutsideLoop,
  ContinueOutsideLoop,
  DerefIncDec(IncDec),
}

#[derive(Debug, Clone, Copy)]
pub enum IncDec {
  Inc,
  Dec,
}

#[derive(Debug, Clone, Copy)]
pub enum Thing {
  Field,
  Struct,
  Variable,
  Function,
  Typedef,
}
