use crate::item::get as get_item;
use crate::util::Cx;
use syntax::ast::{Item, Root};

/// Requires that `root` is lexically and syntactically valid. Ensures this
/// returns `Some(s)` where `s` is the well-formatted text of a C0 program that
/// has the same semantics as `root`.
///
/// Right now this deletes all comments and doesn't care about line length.
///
/// If an invalid `root` is passed, `None` may be returned. Or, `Some(s)` where
/// `s` has different semantics from `root` may also be returned.
pub fn get(root: Root) -> Option<String> {
  let mut cx = Cx::default();
  let mut prev_pragma = true;
  let mut items = root.items();
  if let Some(item) = items.next() {
    prev_pragma = matches!(item, Item::PragmaItem(_));
    get_item(&mut cx, item)?;
  }
  for item in items {
    let this_pragma = matches!(item, Item::PragmaItem(_));
    if !(prev_pragma && this_pragma) {
      cx.push("\n");
    }
    get_item(&mut cx, item)?;
    prev_pragma = this_pragma;
  }
  Some(cx.finish())
}
