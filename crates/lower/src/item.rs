use crate::stmt::get_block;
use crate::ty::get as get_ty;
use crate::util::{Cx, PragmaError};
use hir::Name;
use syntax::ast::{FnTail, Item, Syntax as _};
use syntax::AstPtr;

pub(crate) fn get(
  cx: &mut Cx,
  pragma_ok: &mut bool,
  item: Item,
) -> Option<hir::ItemId> {
  let ptr = AstPtr::new(&item);
  let data = match item {
    Item::StructItem(item) => {
      let name: Name = item.ident()?.text().into();
      // totally ignore `struct foo;`
      let fields: Vec<_> = item
        .fields()?
        .fields()
        .filter_map(|field| {
          let name: Name = field.ident()?.text().into();
          let ty = get_ty(cx, field.ty());
          Some(hir::Field { name, ty })
        })
        .collect();
      *pragma_ok = false;
      hir::Item::Struct(name, fields)
    }
    Item::FnItem(item) => {
      let name: Name = item.ident()?.text().into();
      let params: Vec<_> = item
        .params()
        .filter_map(|param| {
          let name: Name = param.ident()?.text().into();
          let ty = get_ty(cx, param.ty());
          Some(hir::Param { name, ty })
        })
        .collect();
      let ret_ty = get_ty(cx, item.ret_ty());
      let body = match item.tail() {
        None | Some(FnTail::SemicolonTail(_)) => None,
        Some(FnTail::BlockStmt(stmt)) => Some(get_block(cx, stmt)),
      };
      *pragma_ok = false;
      hir::Item::Fn(name, params, ret_ty, body)
    }
    Item::TypedefItem(item) => {
      let name: Name = item.ident()?.text().into();
      let ty = get_ty(cx, item.ty());
      *pragma_ok = false;
      hir::Item::TypeDef(name, ty)
    }
    Item::PragmaItem(item) => {
      if !*pragma_ok {
        cx.errors.push(PragmaError {
          range: item.syntax().text_range(),
        });
      }
      return None;
    }
  };
  let ret = cx.arenas.item.alloc(data);
  cx.ptrs.item.insert(ptr, ret);
  cx.ptrs.item_back.insert(ret, ptr);
  Some(ret)
}
