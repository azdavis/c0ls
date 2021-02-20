use crate::item::get as get_item;
use crate::util::types::{Cx, EnvWithIds, Import};
use hir::Root;

/// `should_define` is whether fns are allowed to have bodies
pub fn get(
  cx: &mut Cx,
  import: &Import,
  should_define: bool,
  root: &Root,
) -> EnvWithIds {
  let mut env = EnvWithIds::default();
  for &item in root.items.iter() {
    get_item(import, &root.arenas, cx, &mut env, should_define, item);
  }
  env
}
