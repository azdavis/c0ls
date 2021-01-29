use crate::error::{ErrorKind, Thing};
use crate::name::Name;
use crate::ty::Ty;
use crate::util::{
  add_var, insert_if_empty, no_struct, unify, Cx, FnData, ItemDb, NameToTy,
  VarDb,
};
use std::collections::hash_map::Entry;
use syntax::ast::{FnItem, FnTail, Item, Syntax};
use syntax::rowan::TextRange;
use unwrap_or::unwrap_or;

pub(crate) fn get(cx: &mut Cx, items: &mut ItemDb, item: Item) {
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
      let (new_data, ranges, mut vars) = get_fn(cx, items, &item);
      let ret_ty = new_data.ret_ty;
      let ident = unwrap_or!(item.ident(), return);
      match items.fns.entry(Name::new(ident.text())) {
        Entry::Occupied(mut entry) => {
          let old_data = entry.get();
          if (old_data.defined && new_data.defined)
            || items.type_defs.contains_key(ident.text())
          {
            cx.error(ident.text_range(), ErrorKind::Duplicate(Thing::Function));
          }
          if old_data.params.len() != new_data.params.len() {
            cx.error(
              ident.text_range(),
              ErrorKind::MismatchedNumParams(
                old_data.params.len(),
                new_data.params.len(),
              ),
            );
          }
          let params = old_data
            .params
            .iter()
            .zip(new_data.params.iter())
            .zip(ranges);
          for ((&(_, old_ty), &(_, new_ty)), range) in params {
            unify(cx, old_ty, Some((range, new_ty)));
          }
          if new_data.defined {
            entry.insert(new_data);
          }
        }
        Entry::Vacant(entry) => {
          entry.insert(new_data);
        }
      }
      if let Some(FnTail::BlockStmt(block)) = item.tail() {
        let range = block.syntax().text_range();
        let ret =
          super::stmt::get_block(cx, items, &mut vars, ret_ty, false, block);
        if ret_ty != Ty::Void && !ret {
          cx.error(range, ErrorKind::InvalidNoReturn);
        }
      }
    }
    Item::TypedefItem(item) => {
      let ident = unwrap_or!(item.ident(), return);
      let text = ident.text();
      let ty = super::ty::get_opt_or(cx, &items.type_defs, item.ty());
      let dup = items.fns.contains_key(text)
        || !insert_if_empty(&mut items.type_defs, Name::new(text), ty);
      if dup {
        cx.error(ident.text_range(), ErrorKind::Duplicate(Thing::Typedef))
      }
    }
    Item::UseItem(_) => todo!("#use and multiple files"),
  }
}

fn get_fn(
  cx: &mut Cx,
  items: &ItemDb,
  item: &FnItem,
) -> (FnData, Vec<TextRange>, VarDb) {
  let mut vars = VarDb::default();
  let mut params = Vec::new();
  let mut ranges = Vec::new();
  for param in item.params() {
    let ty = super::ty::get_opt(cx, &items.type_defs, param.ty());
    if let (Some(ident), Some((ty_range, ty))) = (param.ident(), ty) {
      params.push((Name::new(ident.text()), ty));
      ranges.push(ty_range);
      add_var(cx, &mut vars, &items.type_defs, ident, ty_range, ty, true);
    }
  }
  let ret_ty = match super::ty::get_opt(cx, &items.type_defs, item.ret_ty()) {
    Some((range, ty)) => {
      no_struct(cx, range, ty);
      ty
    }
    None => Ty::Error,
  };
  let defined = match item.tail() {
    None | Some(FnTail::SemicolonTail(_)) => false,
    Some(FnTail::BlockStmt(_)) => true,
  };
  let data = FnData {
    params,
    ret_ty,
    defined,
  };
  (data, ranges, vars)
}
