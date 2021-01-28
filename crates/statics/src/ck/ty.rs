use crate::error::{ErrorKind, Thing};
use crate::name::Name;
use crate::ty::{Ty, TyData};
use crate::util::{Cx, NameToTy};
use syntax::ast::{Syntax as _, Ty as AstTy};
use syntax::rowan::TextRange;

fn get(cx: &mut Cx, type_defs: &NameToTy, ty: AstTy) -> Ty {
  match ty {
    AstTy::IntTy(_) => Ty::Int,
    AstTy::BoolTy(_) => Ty::Bool,
    AstTy::StringTy(_) => Ty::String,
    AstTy::CharTy(_) => Ty::Char,
    AstTy::VoidTy(_) => Ty::Void,
    AstTy::PtrTy(ty) => {
      let inner = get_opt_or(cx, type_defs, ty.ty());
      cx.tys.mk(TyData::Ptr(inner))
    }
    AstTy::ArrayTy(ty) => {
      let inner = get_opt_or(cx, type_defs, ty.ty());
      cx.tys.mk(TyData::Array(inner))
    }
    AstTy::StructTy(ty) => ty.ident().map_or(Ty::Error, |ident| {
      let name = Name::new(ident.text());
      cx.tys.mk(TyData::Struct(name))
    }),
    AstTy::IdentTy(ty) => match ty.ident() {
      None => Ty::Error,
      Some(ident) => match type_defs.get(ident.text()) {
        None => {
          cx.error(ident.text_range(), ErrorKind::Undefined(Thing::Typedef));
          Ty::Error
        }
        Some(&ty) => ty,
      },
    },
  }
}

/// does NOT report an error if it is None, so only call this with optional
/// things from the AST (that have a corresponding parse error).
pub(crate) fn get_opt_or(
  cx: &mut Cx,
  type_defs: &NameToTy,
  ty: Option<AstTy>,
) -> Ty {
  ty.map_or(Ty::Error, |ty| get(cx, type_defs, ty))
}

pub(crate) fn get_opt(
  cx: &mut Cx,
  type_defs: &NameToTy,
  ty: Option<AstTy>,
) -> Option<(TextRange, Ty)> {
  ty.map(|ty| (ty.syntax().text_range(), get(cx, type_defs, ty)))
}
