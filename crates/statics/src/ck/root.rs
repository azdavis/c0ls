use crate::types::{Cx, FileKind, ItemDb};
use syntax::ast::Root;

pub fn get(cx: &mut Cx, items: &mut ItemDb, kind: FileKind, root: Root) {
  let mut pragma_ok = true;
  for item in root.items() {
    if !super::item::get(cx, items, pragma_ok, kind, item) {
      pragma_ok = false;
    }
  }
}
