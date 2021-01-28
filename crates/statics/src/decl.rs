use crate::ty;
use crate::util::error::{ErrorKind, Thing};
use crate::util::{name::Name, no_struct, no_void, ty::Ty, Cx, NameToTy};
use std::collections::hash_map::Entry;
use syntax::{ast::Ty as AstTy, rowan::TextRange, SyntaxToken};

pub(crate) fn get(
  cx: &mut Cx,
  type_defs: &NameToTy,
  vars: &mut NameToTy,
  ident: Option<SyntaxToken>,
  ty: Option<AstTy>,
) -> Option<(TextRange, Ty)> {
  let ret = ty::get_opt(cx, type_defs, ty);
  let ty = match ret {
    None => Ty::Error,
    Some((range, ty)) => {
      no_void(cx, range, ty);
      no_struct(cx, range, ty);
      ty
    }
  };
  if let Some(ident) = ident {
    match vars.entry(Name::new(ident.text())) {
      Entry::Occupied(_) => {
        cx.error(ident.text_range(), ErrorKind::Duplicate(Thing::Variable));
      }
      Entry::Vacant(entry) => {
        entry.insert(ty);
      }
    }
  }
  ret
}
