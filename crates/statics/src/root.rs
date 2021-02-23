use crate::item::get as get_item;
use crate::util::types::{Cx, Env, FileId};
use hir::Root;

pub fn get(cx: &mut Cx, mut env: Env, file: FileId, root: &Root) -> Env {
  for &item in root.items.iter() {
    get_item(&root.arenas, cx, &mut env, file, item);
  }
  env
}
