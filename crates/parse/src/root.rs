use crate::item::item;
use crate::util::TypeDefs;
use syntax::event_parse::Parser;
use syntax::SyntaxKind as SK;

pub(crate) fn root(p: &mut Parser<'_, SK>) {
  let entered = p.enter();
  let mut tds = TypeDefs::new();
  while p.peek().is_some() {
    item(p, &mut tds);
  }
  p.exit(entered, SK::Root);
}
