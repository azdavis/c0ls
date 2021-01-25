use crate::util::name::Name;
use crate::util::ty::TyDb;
use crate::util::{unify, FnData, ItemDb, NameToTy};
use crate::{stmt, ty};
use syntax::ast::{FnItem, FnTail, Item};

pub fn get(items: &mut ItemDb, tys: &mut TyDb, item: Item) -> Option<()> {
  match item {
    Item::StructItem(item) => {
      let fs = match item.fields() {
        Some(fs) => fs,
        // struct decl = do nothing.
        None => return Some(()),
      };
      let mut fields = NameToTy::new();
      for field in fs.fields() {
        let name = Name::new(field.ident()?.text());
        let ty = ty::get(&items.type_defs, tys, field.ty()?)?;
        if fields.insert(name, ty).is_some() {
          return None;
        }
      }
      let name = Name::new(item.ident()?.text());
      if items.structs.insert(name, fields).is_some() {
        return None;
      }
    }
    Item::FnItem(item) => {
      let name = item.ident()?;
      let name = name.text();
      let new_data = get_fn(items, tys, item)?;
      let ins = match items.fns.get(name) {
        None => true,
        Some(old_data) => {
          if old_data.defined && new_data.defined {
            return None;
          }
          if old_data.params.len() != new_data.params.len() {
            return None;
          }
          let params = old_data.params.iter().zip(new_data.params.iter());
          for (&(_, old_ty), &(_, new_ty)) in params {
            unify(tys, old_ty, new_ty)?;
          }
          new_data.defined
        }
      };
      if ins {
        items.fns.insert(Name::new(name), new_data);
      }
    }
    Item::TypedefItem(item) => {
      let name = item.ident()?;
      let name = name.text();
      if items.type_defs.contains_key(name) {
        return None;
      }
      let ty = ty::get(&items.type_defs, tys, item.ty()?)?;
      items.type_defs.insert(Name::new(name), ty);
    }
    Item::UseItem(_) => todo!("#use and multiple files"),
  }
  Some(())
}

fn get_fn(items: &ItemDb, tys: &mut TyDb, item: FnItem) -> Option<FnData> {
  let mut vars = NameToTy::new();
  let mut params = Vec::new();
  for param in item.params() {
    let ty = ty::get(&items.type_defs, tys, param.ty()?)?;
    let name = Name::new(param.ident()?.text());
    if vars.insert(name.clone(), ty).is_some() {
      return None;
    }
    params.push((name, ty));
  }
  let ret_ty = ty::get(&items.type_defs, tys, item.ret_ty()?)?;
  let defined = match item.tail()? {
    FnTail::Semicolon(_) => false,
    FnTail::BlockStmt(block) => {
      if !stmt::get_block(items, &mut vars, tys, ret_ty, block)? {
        return None;
      }
      true
    }
  };
  Some(FnData {
    params,
    ret_ty,
    defined,
  })
}
