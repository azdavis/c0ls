use crate::util::Cx;
use syntax::ast::{AstPtr, Ty};
use unwrap_or::unwrap_or;

pub(crate) fn get(cx: &mut Cx, ty: Option<Ty>) -> hir::TyId {
  let (ptr, data) = ty.map_or((None, hir::Ty::None), |ty| {
    (Some(AstPtr::new(&ty)), get_impl(cx, ty))
  });
  let ret = cx.arenas.ty.alloc(data);
  if let Some(ptr) = ptr {
    cx.ptrs.ty_back.insert(ret, ptr.clone());
    cx.ptrs.ty.insert(ptr, ret);
  }
  ret
}

fn get_impl(cx: &mut Cx, ty: Ty) -> hir::Ty {
  match ty {
    Ty::IntTy(_) => hir::Ty::Int,
    Ty::BoolTy(_) => hir::Ty::Bool,
    Ty::StringTy(_) => hir::Ty::String,
    Ty::CharTy(_) => hir::Ty::Char,
    Ty::VoidTy(_) => hir::Ty::Void,
    Ty::PtrTy(ty) => hir::Ty::Ptr(get(cx, ty.ty())),
    Ty::ArrayTy(ty) => hir::Ty::Array(get(cx, ty.ty())),
    Ty::StructTy(ty) => {
      let name = unwrap_or!(ty.ident(), return hir::Ty::None);
      hir::Ty::Struct(name.text().into())
    }
    Ty::IdentTy(ty) => {
      let name = unwrap_or!(ty.ident(), return hir::Ty::None);
      hir::Ty::Name(name.text().into())
    }
  }
}
