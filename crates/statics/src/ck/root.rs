use crate::util::{Cx, FileKind, ItemDb};
use syntax::ast::Root;

pub fn get(cx: &mut Cx, items: &mut ItemDb, kind: FileKind, root: Root) {
  for item in root.items() {
    super::item::get(cx, items, kind, item);
  }
}
