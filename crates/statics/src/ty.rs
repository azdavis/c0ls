use crate::util::error::ErrorKind;
use crate::util::types::{Cx, Env, Import};
use crate::util::{no_void, ty};
use hir::{Arenas, Ty, TyId};

pub(crate) fn get(
  import: &Import,
  arenas: &Arenas,
  cx: &mut Cx,
  env: &Env,
  ty: TyId,
) -> ty::Ty {
  match arenas.ty[ty] {
    Ty::None => ty::Ty::None,
    Ty::Any => ty::Ty::Any,
    Ty::Int => ty::Ty::Int,
    Ty::Bool => ty::Ty::Bool,
    Ty::Char => ty::Ty::Char,
    Ty::String => ty::Ty::String,
    Ty::Void => ty::Ty::Void,
    Ty::Ptr(ty) => {
      let got_ty = get(import, arenas, cx, env, ty);
      no_void(cx, got_ty, ty);
      cx.tys.mk(ty::TyData::Ptr(got_ty))
    }
    Ty::Array(ty) => {
      let got_ty = get(import, arenas, cx, env, ty);
      no_void(cx, got_ty, ty);
      cx.tys.mk(ty::TyData::Array(got_ty))
    }
    Ty::Struct(ref name) => {
      let data = ty::TyData::Struct(name.clone());
      cx.tys.mk(data)
    }
    Ty::Name(ref name) => {
      if let Some(&type_def) = env.type_defs.get(name) {
        type_def
      } else if let Some(&type_def) = import.type_defs.get(name) {
        type_def
      } else {
        cx.err(ty, ErrorKind::UndefinedTypeDef);
        ty::Ty::None
      }
    }
  }
}
