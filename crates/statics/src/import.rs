use crate::util::error::ErrorKind;
use crate::util::ty::Ty;
use crate::util::types::{Cx, Env, FileId, Import};
use crate::util::unify_impl;

/// TODO this has not great error message IDs (which determine the locations of
/// errors) and duplicates a fair bit of the `Fn` case of `item::get`.
pub fn add_env(
  cx: &mut Cx,
  errors: &mut Vec<ErrorKind>,
  import: &mut Import,
  env: &Env,
  file: FileId,
) {
  for (name, data) in env.fns.iter() {
    let mut sig = data.sig.clone();
    if let Some(old_sig) = import.fns.get(name) {
      let old_sig = old_sig.val();
      let want_len = old_sig.params.len();
      let got_len = sig.params.len();
      if want_len != got_len {
        errors.push(ErrorKind::MismatchedNumParams(want_len, got_len));
      }
      let params_iter = old_sig.params.iter().zip(sig.params.iter());
      for (old, new) in params_iter {
        unify(cx, errors, old.ty, new.ty);
      }
      sig.ret_ty = unify(cx, errors, old_sig.ret_ty, sig.ret_ty);
      sig.should_define = sig.should_define && old_sig.should_define;
      if sig.is_defined && old_sig.is_defined {
        errors.push(ErrorKind::Duplicate(name.clone()));
      }
      if !sig.should_define && sig.is_defined {
        errors.push(ErrorKind::CannotDefnFn)
      }
    }
    import.fns.insert(name.clone(), file.wrap(sig));
  }
  for (name, sig) in env.structs.iter() {
    let val = file.wrap(sig.clone());
    if import.structs.insert(name.clone(), val).is_some() {
      errors.push(ErrorKind::Duplicate(name.clone()));
    }
  }
  for (name, &ty) in env.type_defs.iter() {
    if import
      .type_defs
      .insert(name.clone(), file.wrap(ty))
      .is_some()
    {
      errors.push(ErrorKind::Duplicate(name.clone()));
    }
  }
}

/// records errors in `errors`, not `cx`
fn unify(cx: &mut Cx, errors: &mut Vec<ErrorKind>, want: Ty, got: Ty) -> Ty {
  match unify_impl(&mut cx.tys, want, got) {
    Some(ty) => ty,
    None => {
      errors.push(ErrorKind::MismatchedTys(want, got));
      Ty::None
    }
  }
}
