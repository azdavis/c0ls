use crate::error::{ErrorKind, Thing};
use crate::name::Name;
use crate::ty::Ty;
use crate::util::{
  add_var, insert_if_empty, no_struct, unify, Cx, FnData, ItemDb, NameToTy,
  VarDb,
};
use std::collections::hash_map::Entry;
use syntax::ast::{FnTail, Item, Syntax};
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
          let old_data = entry.get();
          if old_data.params.len() != params.len() {
            cx.error(
              ident.text_range(),
              ErrorKind::MismatchedNumParams(
                old_data.params.len(),
                params.len(),
              ),
            );
          }
          let both_params = old_data.params.iter().zip(params.iter());
          for (&(_, old_ty), &(_, new_ty)) in both_params {
            unify(cx, old_ty, Some(new_ty));
          }
          unify(cx, old_data.ret_ty, ret_ty);
          if tail.is_some() {
            dup = dup || old_data.defined;
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
            defined: tail.is_some(),
          });
        }
      }
      if let Some(block) = tail {
        let range = block.syntax().text_range();
        let ret_ty = ret_ty.map_or(Ty::Error, |x| x.1);
        let mut vars = VarDb::default();
        for (ident, (range, ty)) in params {
          add_var(cx, &mut vars, &items.type_defs, ident, range, ty, true);
        }
        let ret =
          super::stmt::get_block(cx, items, &mut vars, ret_ty, false, block);
        if ret_ty != Ty::Void && !ret {
          cx.error(range, ErrorKind::InvalidNoReturn);
        }
      }
      if dup {
        cx.error(ident.text_range(), ErrorKind::Duplicate(Thing::Function));
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
