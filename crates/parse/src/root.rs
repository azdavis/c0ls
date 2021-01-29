use crate::item::item;
use crate::util::TypeDefs;
use syntax::event_parse::Parser;
use syntax::SyntaxKind as SK;

pub(crate) fn root(p: &mut Parser<'_, SK>, tds: &mut TypeDefs) {
  let entered = p.enter();
  while p.peek().is_some() {
    item(p, tds);
  }
  p.exit(entered, SK::Root);
}
