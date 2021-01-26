use crate::item;
use crate::util::error::Error;
use crate::util::{Cx, ItemDb};
use syntax::ast::Root;

pub fn get(root: Root) -> Vec<Error> {
  let mut items = ItemDb::default();
  let mut cx = Cx::default();
  for item in root.items() {
    item::get(&mut cx, &mut items, item);
  }
  cx.errors.finish()
}
