use crate::stmt::get as get_stmt;
use crate::ty::get as get_ty;
use crate::util::error::ErrorKind;
use crate::util::types::{
  Cx, Defined, Env, FileId, FnCx, FnData, FnSig, Import, InFile, NameToTy,
  Param, VarData,
};
use crate::util::{no_struct, no_unsized, no_void, ty::Ty, unify};
use hir::{Arenas, Item, ItemId};
use uri_db::UriKind;

pub(crate) fn get(
  import: &Import,
  arenas: &Arenas,
  cx: &mut Cx,
  env: &mut Env,
  file: FileId,
  item: ItemId,
) {
  match arenas.item[item] {
    Item::Fn(ref name, ref params, ret_ty, body) => {
      let mut fn_cx = FnCx {
        import,
        arenas,
        vars: Default::default(),
        ret_ty: get_ty(import, arenas, cx, env, ret_ty),
      };
      no_struct(cx, fn_cx.ret_ty, ret_ty);
      let mut sig_params = Vec::with_capacity(params.len());
      for param in params {
        let ty = get_ty(import, arenas, cx, env, param.ty);
        no_void(cx, ty, param.ty);
        no_struct(cx, ty, param.ty);
        let data = VarData { ty, init: true };
        let dup = fn_cx.vars.insert(param.name.clone(), data).is_some()
          || env.type_defs.contains_key(&param.name)
          || import.type_defs.contains_key(&param.name);
        if dup {
          cx.err(item, ErrorKind::Duplicate);
        }
        sig_params.push(Param {
          name: param.name.clone(),
          ty,
        });
      }
      let mut sig = FnSig {
        params: sig_params,
        ret_ty: fn_cx.ret_ty,
        defined: match file {
          FileId::StdLib => Defined::MustNot,
          FileId::Uri(uri) => match uri.kind() {
            UriKind::Header => Defined::MustNot,
            UriKind::Source => {
              if body.is_some() {
                Defined::Yes
              } else {
                Defined::NotYet
              }
            }
          },
        },
      };
      let old_sig = env
        .fns
        .get(name)
        .map(|x| &x.sig)
        .or_else(|| import.fns.get(name).map(InFile::val));
      let mut dup =
        env.type_defs.contains_key(name) || import.type_defs.contains_key(name);
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
        sig.defined = match (old_sig.defined, sig.defined) {
          (Defined::MustNot, _) | (_, Defined::MustNot) => Defined::MustNot,
          (Defined::Yes, Defined::Yes) => {
            dup = true;
            Defined::Yes
          }
          (Defined::Yes, _) | (_, Defined::Yes) => Defined::Yes,
          (Defined::NotYet, Defined::NotYet) => Defined::NotYet,
        }
      }
      if dup {
        cx.err(item, ErrorKind::Duplicate);
      }
      if matches!(sig.defined, Defined::MustNot) && body.is_some() {
        cx.err(item, ErrorKind::DefnHeaderFn)
      }
      let ret_ty = sig.ret_ty;
      env.fns.insert(name.clone(), FnData { sig });
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
        let ty = get_ty(import, arenas, cx, env, field.ty);
        no_unsized(cx, import, env, ty, field.ty);
        if sig.insert(field.name.clone(), ty).is_some() {
          cx.err(field.ty, ErrorKind::Duplicate);
        }
      }
      if env.structs.insert(name.clone(), sig).is_some() {
        cx.err(item, ErrorKind::Duplicate)
      }
    }
    Item::TypeDef(ref name, ty) => {
      let got_ty = get_ty(import, arenas, cx, env, ty);
      no_void(cx, got_ty, ty);
      let dup = env.type_defs.insert(name.clone(), got_ty).is_some()
        || import.type_defs.contains_key(name)
        || env.fns.contains_key(name)
        || import.fns.contains_key(name);
      if dup {
        cx.err(item, ErrorKind::Duplicate)
      }
    }
  }
}
