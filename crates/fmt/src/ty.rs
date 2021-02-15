use crate::util::Cx;
use syntax::ast::Ty;

#[must_use]
pub(crate) fn get(cx: &mut Cx, ty: Ty) -> Option<()> {
  match ty {
    Ty::IntTy(_) => cx.push("int"),
    Ty::BoolTy(_) => cx.push("bool"),
    Ty::StringTy(_) => cx.push("string"),
    Ty::CharTy(_) => cx.push("char"),
    Ty::VoidTy(_) => cx.push("void"),
    Ty::PtrTy(ty) => {
      get(cx, ty.ty()?)?;
      cx.push("*");
    }
    Ty::ArrayTy(ty) => {
      get(cx, ty.ty()?)?;
      cx.push("[]");
    }
    Ty::StructTy(ty) => {
      cx.push("struct ");
      cx.push(ty.ident()?.text());
    }
    Ty::IdentTy(ty) => cx.push(ty.ident()?.text()),
  }
  Some(())
}
