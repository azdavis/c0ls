use crate::item::item;
use crate::Parser;
use syntax::SyntaxKind as SK;

pub(crate) fn root(p: &mut Parser<'_>) {
  let entered = p.enter();
  while p.peek().is_some() {
    item(p);
  }
  p.exit(entered, SK::Root);
}
