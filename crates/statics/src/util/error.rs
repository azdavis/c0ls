use crate::util::ty::Ty;
use syntax::rowan::TextRange;

pub struct Error {
  pub range: TextRange,
  pub kind: ErrorKind,
}

pub enum ErrorKind {
  Duplicate(Thing),
  Undefined(Thing),
  WrongNumParams(usize, usize),
  DerefNull,
  DerefNonPtr(Ty),
  WrongNumArgs(usize, usize),
  SubscriptNonArray(Ty),
  FieldGetNonStruct(Ty),
  NoSuchField,
  MismatchedTypes(Ty, Ty),
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
