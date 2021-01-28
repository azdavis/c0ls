use crate::util::error::{ErrorKind, Thing};
use crate::util::name::Name;
use crate::util::{unify, Cx, FnData, ItemDb, NameToTy};
use crate::{stmt, ty};
use std::collections::hash_map::Entry;
use syntax::ast::{FnItem, FnTail, Item, Syntax};

pub(crate) fn get(cx: &mut Cx, items: &mut ItemDb, item: Item) {
  match item {
    Item::StructItem(item) => {
      let fs = unwrap_or!(item.fields(), return);
      let mut fields = NameToTy::new();
      for field in fs.fields() {
        let ident = unwrap_or!(field.ident(), continue);
        let ty = ty::get_opt_or(cx, &items.type_defs, field.ty());
        match fields.entry(Name::new(ident.text())) {
          Entry::Occupied(_) => cx.errors.push(
            field.syntax().text_range(),
            ErrorKind::Duplicate(Thing::Field),
          ),
          Entry::Vacant(entry) => {
            entry.insert(ty);
          }
        }
      }
      let ident = unwrap_or!(item.ident(), return);
      let name = Name::new(ident.text());
      match items.structs.entry(name) {
        Entry::Occupied(_) => cx
          .errors
          .push(ident.text_range(), ErrorKind::Duplicate(Thing::Struct)),
        Entry::Vacant(entry) => {
          entry.insert(fields);
        }
      }
    }
    Item::FnItem(item) => {
      let new_data = get_fn(cx, items, &item);
      let ident = unwrap_or!(item.ident(), return);
      match items.fns.entry(Name::new(ident.text())) {
        Entry::Occupied(mut entry) => {
          let old_data = entry.get();
          if old_data.defined && new_data.defined {
            cx.errors
              .push(ident.text_range(), ErrorKind::Duplicate(Thing::Function));
          }
          if old_data.params.len() != new_data.params.len() {
            cx.errors.push(
              ident.text_range(),
              ErrorKind::MismatchedNumParams(
                old_data.params.len(),
                new_data.params.len(),
              ),
            );
          }
          let params = old_data.params.iter().zip(new_data.params.iter());
          for (&(_, _, old_ty), &(_, range, new_ty)) in params {
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
    }
    Item::TypedefItem(item) => {
      let ident = unwrap_or!(item.ident(), return);
      let ty = ty::get_opt_or(cx, &items.type_defs, item.ty());
      match items.type_defs.entry(Name::new(ident.text())) {
        Entry::Occupied(_) => cx
          .errors
          .push(ident.text_range(), ErrorKind::Duplicate(Thing::Typedef)),
        Entry::Vacant(entry) => {
          entry.insert(ty);
        }
      }
    }
    Item::UseItem(_) => todo!("#use and multiple files"),
  }
}

fn get_fn(cx: &mut Cx, items: &ItemDb, item: &FnItem) -> FnData {
  let mut vars = NameToTy::new();
  let mut params = Vec::new();
  for param in item.params() {
    let ident = unwrap_or!(param.ident(), continue);
    let (range, ty) =
      unwrap_or!(ty::get_opt(cx, &items.type_defs, param.ty()), continue);
    let name = Name::new(ident.text());
    match vars.entry(name.clone()) {
      Entry::Occupied(_) => cx.errors.push(
        param.syntax().text_range(),
        ErrorKind::Duplicate(Thing::Variable),
      ),
      Entry::Vacant(entry) => {
        entry.insert(ty);
      }
    }
    params.push((name, range, ty));
  }
  let ret_ty = ty::get_opt_or(cx, &items.type_defs, item.ret_ty());
  let defined = match item.tail() {
    None | Some(FnTail::SemicolonTail(_)) => false,
    Some(FnTail::BlockStmt(block)) => {
      if stmt::get_block(cx, items, &mut vars, ret_ty, block) {
        todo!("not all code paths return")
      }
      true
    }
  };
  FnData {
    params,
    ret_ty,
    defined,
  }
}
