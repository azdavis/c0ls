use crate::util::ty::Ty;
use syntax::rowan::TextRange;

pub struct Error {
  pub range: TextRange,
  pub kind: ErrorKind,
}

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
  CannotAssign(Assignment),
}

pub enum Assignment {
  Assign,
  Inc,
  Dec,
}

pub enum Thing {
  Field,
  Struct,
  Variable,
  Function,
  Typedef,
}

#[derive(Default)]
pub(crate) struct ErrorDb {
  inner: Vec<Error>,
}

impl ErrorDb {
  pub(crate) fn push(&mut self, range: TextRange, kind: ErrorKind) {
    self.inner.push(Error { range, kind })
  }

  pub(crate) fn finish(self) -> Vec<Error> {
    self.inner
  }
}
