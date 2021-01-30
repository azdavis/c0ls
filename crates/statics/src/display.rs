use crate::error::ErrorKind;
use crate::ty::{Ty, TyData, TyDb};
use std::fmt;

pub fn error_kind(kind: ErrorKind, tys: &TyDb) -> ErrorKindDisplay<'_> {
  ErrorKindDisplay { kind, tys }
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
          ty(t, self.tys)
        )
      }
      ErrorKind::MismatchedNumArgs(want, got) => write!(
        f,
        "mismatched number of arguments: expected {}, found {}",
        want, got
      ),
      ErrorKind::SubscriptNonArray(t) => {
        write!(f, "cannot subscript non-array type `{}`", ty(t, self.tys))
      }
      ErrorKind::FieldGetNonStruct(t) => write!(
        f,
        "cannot get field of non-struct type `{}`",
        ty(t, self.tys)
      ),
      ErrorKind::MismatchedTys(want, got) => write!(
        f,
        "mismatched types: expected `{}`, found `{}`",
        ty(want, self.tys),
        ty(got, self.tys)
      ),
      ErrorKind::CallNonFn(t) => {
        write!(f, "cannot call non-function type `{}`", ty(t, self.tys))
      }
      ErrorKind::MismatchedTysAny(wants, got) => {
        write!(f, "mismatched types: expected any of ")?;
        for &want in wants.iter() {
          write!(f, "`{}`, ", ty(want, self.tys))?;
        }
        write!(f, "found `{}`", ty(got, self.tys))
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

fn ty(t: Ty, tys: &TyDb) -> TyDisplay<'_> {
  TyDisplay { t, tys }
}

struct TyDisplay<'a> {
  t: Ty,
  tys: &'a TyDb,
}

impl fmt::Display for TyDisplay<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self.tys.get(self.t) {
      TyData::Error => write!(f, "<error>"),
      TyData::Top => write!(f, "<any>"),
      TyData::Int => write!(f, "int"),
      TyData::Bool => write!(f, "bool"),
      TyData::String => write!(f, "string"),
      TyData::Char => write!(f, "char"),
      TyData::Void => write!(f, "void"),
      TyData::Ptr(t) => write!(f, "{}*", ty(t, self.tys)),
      TyData::Array(t) => write!(f, "{}[]", ty(t, self.tys)),
      TyData::Struct(ref name) => write!(f, "struct {}", name),
    }
  }
}
