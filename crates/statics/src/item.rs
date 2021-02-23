use crate::ty::get as get_ty;
use crate::util::error::ErrorKind;
use crate::util::types::{
  Cx, Env, FnCx, FnSig, ItemData, NameToTy, Param, VarData,
};
use crate::util::{no_struct, no_unsized, no_void, ty::Ty, unify};
use crate::{stmt::get as get_stmt, FileId};
use hir::{Arenas, Item, ItemId};

pub(crate) fn get(
  arenas: &Arenas,
  cx: &mut Cx,
  env: &mut Env,
  file: FileId,
  item: ItemId,
) {
  match arenas.item[item] {
    Item::Fn(ref name, ref params, ret_ty, body) => {
      let mut fn_cx = FnCx {
        arenas,
        vars: Default::default(),
        ret_ty: get_ty(arenas, cx, env, ret_ty),
      };
      no_struct(cx, fn_cx.ret_ty, ret_ty);
      let mut sig_params = Vec::with_capacity(params.len());
      for param in params {
        let ty = get_ty(arenas, cx, env, param.ty);
        no_void(cx, ty, param.ty);
        no_struct(cx, ty, param.ty);
        let data = VarData { ty, init: true };
        let dup = fn_cx.vars.insert(param.name.clone(), data).is_some()
          || env.type_defs.contains_key(&param.name);
        if dup {
          cx.err(item, ErrorKind::Duplicate(name.clone()));
        }
        sig_params.push(Param {
          name: param.name.clone(),
          ty,
        });
      }
      let mut sig = FnSig {
        params: sig_params,
        ret_ty: fn_cx.ret_ty,
        is_defined: body.is_some(),
        should_define: matches!(file, FileId::Source(_)),
      };
      let old_sig = env.fns.get(name).map(ItemData::val);
      let mut dup = env.type_defs.contains_key(name);
      if let Some(old_sig) = old_sig {
        let want_len = old_sig.params.len();
        let got_len = sig.params.len();
        if want_len != got_len {
          cx.err(item, ErrorKind::MismatchedNumParams(want_len, got_len));
        }
        let params_iter = old_sig
          .params
          .iter()
          .zip(sig.params.iter())
          .zip(params.iter());
        for ((old, new), p) in params_iter {
          unify(cx, old.ty, new.ty, p.ty);
        }
        sig.ret_ty = unify(cx, old_sig.ret_ty, sig.ret_ty, ret_ty);
        sig.should_define = sig.should_define && old_sig.should_define;
        dup = dup || (sig.is_defined && old_sig.is_defined);
      }
      if dup {
        cx.err(item, ErrorKind::Duplicate(name.clone()));
      }
      if !sig.should_define && sig.is_defined {
        cx.err(item, ErrorKind::CannotDefnFn)
      }
      let ret_ty = sig.ret_ty;
      env.fns.insert(name.clone(), ItemData::new(file, item, sig));
      if let Some(body) = body {
        let diverges = get_stmt(cx, env, &mut fn_cx, false, body);
        if !diverges && ret_ty != Ty::Void {
          cx.err(body, ErrorKind::FnMightNotReturnVal);
        }
      }
    }
    Item::Struct(ref name, ref fields) => {
      let mut sig = NameToTy::default();
      for field in fields {
        let ty = get_ty(arenas, cx, env, field.ty);
        no_unsized(cx, env, ty, field.ty);
        if sig.insert(field.name.clone(), ty).is_some() {
          cx.err(field.ty, ErrorKind::Duplicate(name.clone()));
        }
      }
      if env
        .structs
        .insert(name.clone(), ItemData::new(file, item, sig))
        .is_some()
      {
        cx.err(item, ErrorKind::Duplicate(name.clone()))
      }
    }
    Item::TypeDef(ref name, ty) => {
      let got_ty = get_ty(arenas, cx, env, ty);
      no_void(cx, got_ty, ty);
      let dup = env
        .type_defs
        .insert(name.clone(), ItemData::new(file, item, got_ty))
        .is_some()
        || env.fns.contains_key(name);
      if dup {
        cx.err(item, ErrorKind::Duplicate(name.clone()))
      }
    }
  }
}
