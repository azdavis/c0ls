use crate::item::get as get_item;
use crate::util::file::FileKind;
use crate::util::types::{Cx, Env, Import};
use hir::Root;

pub fn get(cx: &mut Cx, import: &Import, kind: FileKind, root: &Root) -> Env {
  let mut env = Env::default();
  for &item in root.items.iter() {
    get_item(import, &root.arenas, cx, &mut env, kind, item);
  }
  env
}
