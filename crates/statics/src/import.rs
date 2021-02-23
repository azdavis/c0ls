use crate::util::error::ErrorKind;
use crate::util::ty::Ty;
use crate::util::types::{Cx, Env};
use crate::util::unify_impl;

/// TODO this has not great error message IDs (which determine the locations of
/// errors) and duplicates a fair bit of the `Fn` case of `item::get`.
pub fn add_env(
  cx: &mut Cx,
  errors: &mut Vec<ErrorKind>,
  import: &mut Env,
  env: &Env,
) {
  for (name, sig) in env.fns.iter() {
    let mut sig = sig.clone();
    if let Some(old_sig) = import.fns.get(name) {
      let old_sig = old_sig.val();
      let want_len = old_sig.params.len();
      let got_len = sig.val().params.len();
      if want_len != got_len {
        errors.push(ErrorKind::MismatchedNumParams(want_len, got_len));
      }
      let params_iter = old_sig.params.iter().zip(sig.val().params.iter());
      for (old, new) in params_iter {
        unify(cx, errors, old.ty, new.ty);
      }
      sig.val_mut().ret_ty =
        unify(cx, errors, old_sig.ret_ty, sig.val().ret_ty);
      sig.val_mut().should_define =
        sig.val().should_define && old_sig.should_define;
      if sig.val().is_defined && old_sig.is_defined {
        errors.push(ErrorKind::Duplicate(name.clone()));
      }
      if !sig.val().should_define && sig.val().is_defined {
        errors.push(ErrorKind::CannotDefnFn)
      }
    }
    import.fns.insert(name.clone(), sig);
  }
  for (name, sig) in env.structs.iter() {
    if import.structs.insert(name.clone(), sig.clone()).is_some() {
      errors.push(ErrorKind::Duplicate(name.clone()));
    }
  }
  for (name, &ty) in env.type_defs.iter() {
    if import.type_defs.insert(name.clone(), ty).is_some() {
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
