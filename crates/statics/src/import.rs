use crate::util::error::ErrorKind;
use crate::util::types::{Cx, EnvWithIds, FileId, Import};
use crate::util::unify;

/// TODO this has not great error message IDs (which determine the locations of
/// errors) and duplicates a fair bit of the `Fn` case of `item::get`.
pub fn add_env(
  cx: &mut Cx,
  import: &mut Import,
  env: &EnvWithIds,
  file: FileId,
) {
  for (name, data) in env.env.fns.iter() {
    let mut sig = data.sig.clone();
    let item = env.ids.fns[name];
    if let Some(old_sig) = import.fns.get(name) {
      let old_sig = old_sig.val();
      let want_len = old_sig.params.len();
      let got_len = sig.params.len();
      if want_len != got_len {
        cx.err(item, ErrorKind::MismatchedNumParams(want_len, got_len));
      }
      let params_iter = old_sig.params.iter().zip(sig.params.iter());
      for (old, new) in params_iter {
        unify(cx, old.ty, new.ty, item);
      }
      sig.ret_ty = unify(cx, old_sig.ret_ty, sig.ret_ty, item);
      sig.should_define = sig.should_define && old_sig.should_define;
      if sig.is_defined && old_sig.is_defined {
        cx.err(item, ErrorKind::Duplicate(name.clone()));
      }
      if !sig.should_define && sig.is_defined {
        cx.err(item, ErrorKind::CannotDefnFn)
      }
    }
    import.fns.insert(name.clone(), file.wrap(sig));
  }
  for (name, sig) in env.env.structs.iter() {
    let item = env.ids.structs[name];
    let val = file.wrap(sig.clone());
    if import.structs.insert(name.clone(), val).is_some() {
      cx.err(item, ErrorKind::Duplicate(name.clone()));
    }
  }
  for (name, &ty) in env.env.type_defs.iter() {
    let item = env.ids.type_defs[name];
    if import
      .type_defs
      .insert(name.clone(), file.wrap(ty))
      .is_some()
    {
      cx.err(item, ErrorKind::Duplicate(name.clone()));
    }
  }
}
