use crate::error::{ErrorKind, Thing};
use crate::name::Name;
use crate::ty::{Ty, TyData};
use crate::util::{no_void, Cx, ItemDb, NameToTy};
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
      let inner = get_opt_no_void(cx, type_defs, ty.ty());
      cx.tys.mk(TyData::Ptr(inner))
    }
    AstTy::ArrayTy(ty) => {
      let inner = get_opt_no_void(cx, type_defs, ty.ty());
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

pub(crate) fn get_opt(
  cx: &mut Cx,
  type_defs: &NameToTy,
  ty: Option<AstTy>,
) -> Option<(TextRange, Ty)> {
  ty.map(|ty| (ty.syntax().text_range(), get(cx, type_defs, ty)))
}

/// does NOT report an error if it is None, so only call this with optional
/// things from the AST (that have a corresponding parse error). also errors if
/// the ty is void.
pub(crate) fn get_opt_no_void(
  cx: &mut Cx,
  type_defs: &NameToTy,
  ty: Option<AstTy>,
) -> Ty {
  get_opt(cx, type_defs, ty).map_or(Ty::Error, |(range, ty)| {
    no_void(cx, range, ty);
    ty
  })
}

/// use this when we would need to know the size of a type on the stack. this
/// does extra checks to make sure we know the size of the type that would be
/// returned.
pub(crate) fn get_sized_opt_or(
  cx: &mut Cx,
  items: &ItemDb,
  ty: Option<AstTy>,
) -> Ty {
  get_opt(cx, &items.type_defs, ty).map_or(Ty::Error, |(range, ty)| {
    no_void(cx, range, ty);
    if let TyData::Struct(name) = cx.tys.get(ty) {
      if !items.structs.contains_key(name) {
        cx.error(range, ErrorKind::Undefined(Thing::Struct))
      }
    }
    ty
  })
}
