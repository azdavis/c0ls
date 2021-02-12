use crate::item::get as get_item;
use crate::util::{Cx, Lowered};
use syntax::ast::Root;

pub fn get(root: Root) -> Lowered {
  let mut cx = Cx::default();
  let mut items = Vec::new();
  let mut pragma_ok = true;
  for item in root.items() {
    if let Some(item) = get_item(&mut cx, &mut pragma_ok, item) {
      items.push(item)
    }
  }
  cx.arenas.item.shrink_to_fit();
  cx.arenas.expr.shrink_to_fit();
  cx.arenas.ty.shrink_to_fit();
  cx.arenas.stmt.shrink_to_fit();
  cx.arenas.simp.shrink_to_fit();
  Lowered {
    root: hir::Root {
      arenas: cx.arenas,
      items,
    },
    ptrs: cx.ptrs,
    errors: cx.errors,
  }
}
