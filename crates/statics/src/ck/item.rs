use crate::error::{ErrorKind, Thing};
use crate::name::Name;
use crate::ty::Ty;
use crate::util::{
  add_var, insert_if_empty, no_struct, no_void, unify, Cx, FileKind, FnData,
  ItemDb, NameToTy, VarDb,
};
use std::collections::hash_map::Entry;
use syntax::ast::{FnTail, Item, Syntax};
use unwrap_or::unwrap_or;

pub(crate) fn get(cx: &mut Cx, items: &mut ItemDb, kind: FileKind, item: Item) {
  match item {
    Item::StructItem(item) => {
      let fs = unwrap_or!(item.fields(), return);
      let mut fields = NameToTy::default();
      for field in fs.fields() {
        let ident = unwrap_or!(field.ident(), continue);
        let ty = super::ty::get_sized_opt_or(cx, items, field.ty());
        if !insert_if_empty(&mut fields, Name::new(ident.text()), ty) {
          cx.error(
            field.syntax().text_range(),
            ErrorKind::Duplicate(Thing::Field),
          )
        }
      }
      let ident = unwrap_or!(item.ident(), return);
      let name = Name::new(ident.text());
      if !insert_if_empty(&mut items.structs, name, fields) {
        cx.error(ident.text_range(), ErrorKind::Duplicate(Thing::Struct))
      }
    }
    Item::FnItem(item) => {
      let params: Vec<_> = item
        .params()
        .flat_map(|param| {
          let ty = super::ty::get_opt(cx, &items.type_defs, param.ty());
          param.ident().zip(ty)
        })
        .collect();
      let ret_ty = super::ty::get_opt(cx, &items.type_defs, item.ret_ty());
      if let Some((range, ty)) = ret_ty {
        no_struct(cx, range, ty);
      }
      let tail = match item.tail() {
        None | Some(FnTail::SemicolonTail(_)) => None,
        Some(FnTail::BlockStmt(block)) => Some(block),
      };
      let ident = unwrap_or!(item.ident(), return);
      let mut dup = items.type_defs.contains_key(ident.text());
      match items.fns.entry(Name::new(ident.text())) {
        Entry::Occupied(mut entry) => {
          let old = entry.get();
          if old.params.len() != params.len() {
            cx.error(
              ident.text_range(),
              ErrorKind::MismatchedNumParams(old.params.len(), params.len()),
            );
          }
          let both_params = old.params.iter().zip(params.iter());
          for (&(_, old_ty), &(_, new_ty)) in both_params {
            unify(cx, old_ty, Some(new_ty));
          }
          unify(cx, old.ret_ty, ret_ty);
          if tail.is_some() {
            dup = dup || old.defined;
            entry.get_mut().defined = true;
          }
        }
        Entry::Vacant(entry) => {
          entry.insert(FnData {
            params: params
              .iter()
              .map(|&(ref n, (_, t))| (Name::new(n.text()), t))
              .collect(),
            ret_ty: ret_ty.map_or(Ty::Error, |x| x.1),
            defined: tail.is_some() || matches!(kind, FileKind::Header),
          });
        }
      }
      // put this here, and not in the `if let`, so we check the param types are
      // valid even if this is just a decl.
      let mut vars = VarDb::default();
      for (ident, (range, ty)) in params {
        add_var(cx, &mut vars, &items.type_defs, ident, range, ty, true);
      }
      if let Some(block) = tail {
        let range = block.syntax().text_range();
        let ret_ty = ret_ty.map_or(Ty::Error, |x| x.1);
        let ret =
          super::stmt::get_block(cx, items, &mut vars, ret_ty, false, block);
        if ret_ty != Ty::Void && !ret {
          cx.error(range, ErrorKind::InvalidNoReturn);
        }
        if matches!(kind, FileKind::Header) {
          cx.error(range, ErrorKind::DefnInHeader);
        }
      }
      if dup {
        cx.error(ident.text_range(), ErrorKind::Duplicate(Thing::Function));
      }
    }
    Item::TypedefItem(item) => {
      let ty = match super::ty::get_opt(cx, &items.type_defs, item.ty()) {
        Some((range, ty)) => {
          no_void(cx, range, ty);
          ty
        }
        None => Ty::Error,
      };
      let ident = unwrap_or!(item.ident(), return);
      let text = ident.text();
      let dup = items.fns.contains_key(text)
        || !insert_if_empty(&mut items.type_defs, Name::new(text), ty);
      if dup {
        cx.error(ident.text_range(), ErrorKind::Duplicate(Thing::Typedef))
      }
    }
    Item::UseItem(_) => todo!("#use and multiple files"),
  }
}
