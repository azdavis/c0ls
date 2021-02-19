use crate::db::Db;
use crate::types::{CodeBlock, Hover};
use crate::util::get_token;
use lower::AstPtr;
use statics::InFile;
use syntax::ast::{Cast as _, Expr, Syntax as _};
use text_pos::Position;
use uri_db::Uri;

pub(crate) fn get(db: &Db, uri: &Uri, pos: Position) -> Option<Hover> {
  let done = db.kind.done()?;
  let id = db.uris.get_id(uri)?;
  let syntax_data = &db.syntax_data[&id];
  let mut node = get_token(syntax_data, pos)?.parent();
  let expr_node = loop {
    match Expr::cast(node.clone().into()) {
      Some(x) => break x,
      None => node = node.parent()?,
    }
  };
  let expr = *syntax_data.ptrs.expr.get(&AstPtr::new(&expr_node))?;
  let range = syntax_data.positions.range(expr_node.syntax().text_range());
  let semantic_data = &done.semantic_data[&id];
  let contents = match syntax_data.hir_root.arenas.expr[expr] {
    hir::Expr::Call(ref name, _) => semantic_data
      .import
      .fns
      .get(name)
      .map(InFile::val)
      .or_else(|| semantic_data.env.env.fns.get(name).map(|x| &x.sig))?
      .display(name, &done.cx.tys)
      .to_string(),
    _ => semantic_data
      .env
      .env
      .expr_tys
      .get(expr)?
      .display(&done.cx.tys)
      .to_string(),
  };
  Some(Hover {
    contents: CodeBlock::new(contents),
    range,
  })
}
