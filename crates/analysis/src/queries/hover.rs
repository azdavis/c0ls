use crate::db::Db;
use crate::types::{CodeBlock, Hover};
use crate::util::get_token;
use std::convert::TryFrom;
use syntax::ast::{Expr, Ty};
use syntax::AstPtr;
use text_pos::Position;
use uri_db::Uri;

pub(crate) fn get(db: &Db, uri: &Uri, pos: Position) -> Option<Hover> {
  let done = db.kind.done()?;
  let id = db.uris.get_id(uri)?;
  let syntax_data = &db.syntax_data[&id];
  let semantic_data = &done.semantic_data[&id];
  let mut node = get_token(syntax_data, pos)?.parent();
  loop {
    if let Ok(expr_node) = Expr::try_from(node.clone()) {
      let expr = *syntax_data.ptrs.expr.get(&AstPtr::new(&expr_node))?;
      let contents = match syntax_data.hir_root.arenas.expr[expr] {
        hir::Expr::Call(ref name, _) => semantic_data
          .env
          .fns
          .get(name)?
          .val()
          .display(name, &done.cx.tys)
          .to_string(),
        _ => semantic_data
          .env
          .expr_tys
          .get(expr)?
          .display(&done.cx.tys)
          .to_string(),
      };
      return Some(Hover {
        contents: CodeBlock::new(contents),
        range: syntax_data.positions.range(expr_node.as_ref().text_range()),
      });
    }
    if let Ok(ty_node) = Ty::try_from(node.clone()) {
      let ty = *syntax_data.ptrs.ty.get(&AstPtr::new(&ty_node))?;
      return Some(Hover {
        contents: CodeBlock::new(
          semantic_data
            .env
            .ty_tys
            .get(ty)?
            .display(&done.cx.tys)
            .to_string(),
        ),
        range: syntax_data.positions.range(ty_node.as_ref().text_range()),
      });
    }
    node = node.parent()?;
  }
}
