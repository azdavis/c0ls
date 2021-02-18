use crate::item::get as get_item;
use crate::util::types::{Cx, Env, FileId, Import};
use hir::Root;

pub fn get(cx: &mut Cx, import: &Import, file: FileId, root: &Root) -> Env {
  let mut env = Env::default();
  for &item in root.items.iter() {
    get_item(import, &root.arenas, cx, &mut env, file, item);
  }
  env
}
