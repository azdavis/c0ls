use crate::item;
use crate::util::ty::TyDb;
use crate::util::ItemDb;
use syntax::ast::Root;

pub fn get(root: Root) -> Option<()> {
  let mut items = ItemDb::default();
  let mut tys = TyDb::default();
  for item in root.items() {
    item::get(&mut items, &mut tys, item)?;
  }
  Some(())
}
