use crate::item::get as get_item;
use crate::util::types::{Cx, EnvWithIds, FileId, Import};
use hir::Root;

pub fn get(
  cx: &mut Cx,
  import: &Import,
  file: FileId,
  root: &Root,
) -> EnvWithIds {
  let mut env = EnvWithIds::default();
  for &item in root.items.iter() {
    get_item(import, &root.arenas, cx, &mut env, file, item);
  }
  env
}
