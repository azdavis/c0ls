use crate::util::id::Id;
use crate::util::ty::{Ty, TyDb};
use hir::Name;
use std::fmt;

#[derive(Debug)]
pub struct Error {
  pub id: Id,
  pub kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
  CallNonFnTy(Ty),
  CannotAssign,
  CannotIncDec(hir::IncDec),
  DeclInForStep,
  DefnHeaderFn,
  DerefNonPtrTy(Ty),
  DerefNull,
  Duplicate(Name),
  FieldGetNonStructTy(Ty),
  FnMightNotReturnVal,
  InvalidStructTy,
  InvalidVoidTy,
  MismatchedNumArgs(usize, usize),
  MismatchedNumParams(usize, usize),
  MismatchedTys(Ty, Ty),
  MismatchedTysAny(&'static [Ty], Ty),
  NotInLoop,
  ReturnExprVoid,
  ReturnNothingNonVoid(Ty),
  SubscriptNonArrayTy(Ty),
  UndefinedField(Name),
  UndefinedFn(Name),
  UndefinedStruct(Name),
  UndefinedTypeDef(Name),
  UndefinedVar(Name),
  UninitializedVar(Name),
}

impl ErrorKind {
  pub fn display<'a>(&'a self, tys: &'a TyDb) -> ErrorKindDisplay<'a> {
    ErrorKindDisplay { this: self, tys }
  }
}

#[derive(Debug)]
pub struct ErrorKindDisplay<'a> {
  this: &'a ErrorKind,
  tys: &'a TyDb,
}

impl fmt::Display for ErrorKindDisplay<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.this {
      ErrorKind::CallNonFnTy(t) => {
        write!(f, "cannot call non-function type `{}`", t.display(self.tys))
      }
      ErrorKind::CannotAssign => write!(f, "cannot assign to this expression"),
      ErrorKind::CannotIncDec(x) => write!(f, "cannot {} this expression", x),
      ErrorKind::DeclInForStep => {
        write!(f, "cannot declare a variable in `for` loop step")
      }
      ErrorKind::DefnHeaderFn => write!(f, "cannot define a header function"),
      ErrorKind::DerefNonPtrTy(t) => write!(
        f,
        "cannot dereference non-pointer type `{}`",
        t.display(self.tys)
      ),
      ErrorKind::DerefNull => write!(f, "cannot dereference `NULL`"),
      ErrorKind::Duplicate(name) => {
        write!(f, "duplicate definitions for `{}`", name)
      }
      ErrorKind::FieldGetNonStructTy(t) => write!(
        f,
        "cannot get field of non-struct type `{}`",
        t.display(self.tys)
      ),
      ErrorKind::FnMightNotReturnVal => {
        write!(f, "cannot reach end of function without returning a value")
      }
      ErrorKind::InvalidStructTy => write!(f, "cannot use struct type here"),
      ErrorKind::InvalidVoidTy => write!(f, "cannot use void type here"),
      ErrorKind::MismatchedNumArgs(want, got) => write!(
        f,
        "mismatched number of arguments: expected {}, found {}",
        want, got
      ),
      ErrorKind::MismatchedNumParams(want, got) => write!(
        f,
        "mismatched number of parameters: expected {}, found {}",
        want, got
      ),
      ErrorKind::MismatchedTys(want, got) => write!(
        f,
        "mismatched types: expected `{}`, found `{}`",
        want.display(self.tys),
        got.display(self.tys)
      ),
      ErrorKind::MismatchedTysAny(wants, got) => {
        assert!(wants.len() >= 2);
        let mut wants = wants.iter();
        write!(
          f,
          "mismatched types: expected any of {{`{}`",
          wants.next().unwrap().display(self.tys)
        )?;
        for want in wants {
          write!(f, ", `{}`", want.display(self.tys))?;
        }
        write!(f, "}}, found `{}`", got.display(self.tys))
      }
      ErrorKind::NotInLoop => {
        write!(f, "cannot use this statement outside of a loop")
      }
      ErrorKind::ReturnExprVoid => {
        write!(f, "cannot return a value from a function returning `void`")
      }
      ErrorKind::ReturnNothingNonVoid(t) => write!(
        f,
        "cannot return without a value from a function returning `{}`",
        t.display(self.tys)
      ),
      ErrorKind::SubscriptNonArrayTy(t) => write!(
        f,
        "cannot subscript non-array type `{}`",
        t.display(self.tys)
      ),
      ErrorKind::UndefinedField(name) => {
        write!(f, "undefined field `{}`", name)
      }
      ErrorKind::UndefinedFn(name) => {
        write!(f, "undefined function `{}`", name)
      }
      ErrorKind::UndefinedStruct(name) => {
        write!(f, "undefined struct `{}`", name)
      }
      ErrorKind::UndefinedTypeDef(name) => {
        write!(f, "undefined typedef `{}`", name)
      }
      ErrorKind::UndefinedVar(name) => {
        write!(f, "undefined variable `{}`", name)
      }
      ErrorKind::UninitializedVar(name) => {
        write!(f, "uninitialized variable `{}`", name)
      }
    }
  }
}
