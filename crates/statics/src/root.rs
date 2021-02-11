use crate::item::get as get_item;
use crate::util::file::FileId;
use crate::util::types::{Cx, Env, Import};
use hir::Root;

pub fn get(cx: &mut Cx, import: &Import, id: FileId, root: &Root) -> Env {
  let mut env = Env::default();
  let kind = id.kind();
  for &item in root.items.iter() {
    get_item(import, &root.arenas, cx, &mut env, kind, item);
  }
  env
}
