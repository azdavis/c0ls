use crate::util::{Cx, ItemDb};
use syntax::ast::Root;

pub fn get(cx: &mut Cx, items: &mut ItemDb, root: Root) {
  for item in root.items() {
    super::item::get(cx, items, item);
  }
}
