use crate::util::error::ErrorKind;
use crate::util::types::{Cx, Env};
use crate::util::{no_void, ty};
use hir::{Arenas, Ty, TyId};

pub(crate) fn get(
  arenas: &Arenas,
  cx: &mut Cx,
  env: &mut Env,
  ty: TyId,
) -> ty::Ty {
  let ret = match arenas.ty[ty] {
    Ty::None => ty::Ty::None,
    Ty::Any => ty::Ty::Any,
    Ty::Int => ty::Ty::Int,
    Ty::Bool => ty::Ty::Bool,
    Ty::Char => ty::Ty::Char,
    Ty::String => ty::Ty::String,
    Ty::Void => ty::Ty::Void,
    Ty::Ptr(ty) => {
      let got_ty = get(arenas, cx, env, ty);
      no_void(cx, got_ty, ty);
      cx.tys.mk(ty::TyData::Ptr(got_ty))
    }
    Ty::Array(ty) => {
      let got_ty = get(arenas, cx, env, ty);
      no_void(cx, got_ty, ty);
      cx.tys.mk(ty::TyData::Array(got_ty))
    }
    Ty::Struct(ref name) => {
      let data = ty::TyData::Struct(name.clone());
      cx.tys.mk(data)
    }
    Ty::Name(ref name) => {
      if let Some(type_def) = env.type_defs.get(name) {
        *type_def.val()
      } else {
        cx.err(ty, ErrorKind::UndefinedTypeDef(name.clone()));
        ty::Ty::None
      }
    }
  };
  env.ty_tys.insert(ty, ret);
  ret
}
