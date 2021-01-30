use crate::ty::{Ty, TyDb};
use std::fmt;
use syntax::rowan::TextRange;

#[derive(Debug)]
pub struct Error {
  pub range: TextRange,
  pub kind: ErrorKind,
}

#[derive(Debug, Clone, Copy)]
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

impl ErrorKind {
  pub fn display(self, tys: &TyDb) -> ErrorKindDisplay<'_> {
    ErrorKindDisplay { kind: self, tys }
  }
}

#[derive(Debug)]
pub struct ErrorKindDisplay<'a> {
  kind: ErrorKind,
  tys: &'a TyDb,
}

impl fmt::Display for ErrorKindDisplay<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self.kind {
      ErrorKind::Duplicate(thing) => {
        write!(f, "duplicate definitions for {}", thing.display())
      }
      ErrorKind::Undefined(thing) => write!(f, "undefined {}", thing.display()),
      ErrorKind::MismatchedNumParams(want, got) => write!(
        f,
        "mismatched number of parameters: expected {}, found {}",
        want, got
      ),
      ErrorKind::DerefNull => write!(f, "cannot dereference NULL"),
      ErrorKind::DerefNonPtr(t) => {
        write!(
          f,
          "cannot dereference non-pointer type `{}`",
          t.display(self.tys)
        )
      }
      ErrorKind::MismatchedNumArgs(want, got) => write!(
        f,
        "mismatched number of arguments: expected {}, found {}",
        want, got
      ),
      ErrorKind::SubscriptNonArray(t) => {
        write!(
          f,
          "cannot subscript non-array type `{}`",
          t.display(self.tys)
        )
      }
      ErrorKind::FieldGetNonStruct(t) => write!(
        f,
        "cannot get field of non-struct type `{}`",
        t.display(self.tys)
      ),
      ErrorKind::MismatchedTys(want, got) => write!(
        f,
        "mismatched types: expected `{}`, found `{}`",
        want.display(self.tys),
        got.display(self.tys)
      ),
      ErrorKind::CallNonFn(t) => {
        write!(f, "cannot call non-function type `{}`", t.display(self.tys))
      }
      ErrorKind::MismatchedTysAny(wants, got) => {
        write!(f, "mismatched types: expected any of ")?;
        for &want in wants.iter() {
          write!(f, "`{}`, ", want.display(self.tys))?;
        }
        write!(f, "found `{}`", got.display(self.tys))
      }
      ErrorKind::InvalidVoidTy => write!(f, "cannot use void type here"),
      ErrorKind::InvalidStructTy => write!(f, "cannot use struct type here"),
      ErrorKind::InvalidNoReturn => write!(
        f,
        "control reaches end of non-void function without returning"
      ),
      ErrorKind::InvalidStepDecl => {
        write!(f, "cannot declare a variable in `for` loop step")
      }
      ErrorKind::ReturnExprVoid => {
        write!(f, "cannot return an expression from a void function")
      }
      ErrorKind::NoReturnExprNotVoid => write!(
        f,
        "cannot return without an expression from a non-void function"
      ),
      ErrorKind::InvalidAssign(inc_dec) => match inc_dec {
        None => write!(f, "cannot assign to this expression"),
        Some(x) => write!(f, "cannot {} this expression", x.display()),
      },
      ErrorKind::UninitializedVar => {
        write!(f, "cannot use uninitialized variable here")
      }
      ErrorKind::BreakOutsideLoop => write!(f, "cannot break outside of loop"),
      ErrorKind::ContinueOutsideLoop => {
        write!(f, "cannot continue outside of loop")
      }
      ErrorKind::DerefIncDec(x) => write!(
        f,
        "cannot {} a dereference without parentheses",
        x.display()
      ),
      ErrorKind::DefnOfHeaderFn => write!(f, "cannot define a header function"),
      ErrorKind::PragmaNotFirst => {
        write!(f, "pragmas must come before all other items")
      }
    }
  }
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
