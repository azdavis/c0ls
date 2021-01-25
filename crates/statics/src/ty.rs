use crate::util::name::Name;
use crate::util::ty::{Ty, TyData, TyDb};
use crate::util::NameToTy;
use syntax::ast::Ty as AstTy;

pub fn get(type_defs: &NameToTy, tys: &mut TyDb, ty: AstTy) -> Option<Ty> {
  match ty {
    AstTy::IntTy(_) => Some(Ty::Int),
    AstTy::BoolTy(_) => Some(Ty::Bool),
    AstTy::StringTy(_) => Some(Ty::String),
    AstTy::CharTy(_) => Some(Ty::Char),
    AstTy::VoidTy(_) => Some(Ty::Void),
    AstTy::PtrTy(ty) => {
      let inner = get(type_defs, tys, ty.ty()?)?;
      Some(tys.mk(TyData::Ptr(inner)))
    }
    AstTy::ArrayTy(ty) => {
      let inner = get(type_defs, tys, ty.ty()?)?;
      Some(tys.mk(TyData::Array(inner)))
    }
    AstTy::StructTy(ty) => {
      let name = Name::new(ty.ident()?.text());
      Some(tys.mk(TyData::Struct(name)))
    }
    AstTy::IdentTy(ty) => type_defs.get(ty.ident()?.text()).copied(),
  }
}
