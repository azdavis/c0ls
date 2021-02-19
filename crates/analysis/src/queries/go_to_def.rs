use crate::db::{Db, SemanticData};
use crate::types::Location;
use crate::util::get_token;
use lower::AstPtr;
use rustc_hash::FxHashMap;
use statics::{InFile, TyData};
use syntax::ast::{Cast as _, Expr, Simp, Stmt, Syntax as _, Ty};
use syntax::SyntaxKind;
use text_pos::Position;
use uri_db::{Uri, UriId};

pub(crate) fn get(db: &Db, uri: &Uri, pos: Position) -> Option<Location> {
  let done = db.kind.done()?;
  let id = db.uris.get_id(uri)?;
  let syntax_data = &db.syntax_data[&id];
  let tok = get_token(syntax_data, pos)?;
  if tok.kind() != SyntaxKind::Ident {
    return None;
  }
  let semantic_data = &done.semantic_data[&id];
  if let Some(expr) = Expr::cast(tok.parent().into()) {
    let expr = syntax_data.ptrs.expr[&AstPtr::new(&expr)];
    match syntax_data.hir_root.arenas.expr[expr] {
      hir::Expr::Name(ref name) => {
        // find the stmt containing this expr. we know tok.parent() is an expr,
        // so go up again to start off.
        let mut node = tok.parent().parent()?;
        let mut stmt_node = loop {
          match Stmt::cast(node.clone().into()) {
            Some(x) => break x,
            None => node = node.parent()?,
          }
        };
        // work backward up the body of the fn, checking each decl.
        let range = loop {
          if let Stmt::SimpStmt(ref stmt) = stmt_node {
            let tok = match stmt.simp() {
              Some(Simp::DeclSimp(simp)) => simp.ident(),
              Some(Simp::AmbiguousSimp(simp)) => simp.lhs(),
              _ => None,
            };
            if tok.map_or(false, |tok| name == tok.text()) {
              break syntax_data.positions.range(stmt.syntax().text_range());
            }
          }
          // might have a prev sibling be an expr (if we're e.g. in an IfStmt)
          stmt_node = stmt_node
            .syntax()
            .prev_sibling()
            .and_then(|x| Stmt::cast(x.into()))
            .or_else(|| Stmt::cast(stmt_node.syntax().parent()?.into()))?;
        };
        Some(Location {
          uri: uri.clone(),
          range,
        })
      }
      hir::Expr::Call(ref name, _) => get_item_loc(
        db,
        &semantic_data.import.fns,
        &semantic_data.env.env.fns,
        id,
        name,
        |item| match *item {
          hir::Item::Fn(ref the_name, _, _, _) => the_name == name,
          _ => false,
        },
      ),
      hir::Expr::FieldGet(expr, _) => {
        match done.cx.tys.get(semantic_data.env.env.expr_tys[expr]) {
          TyData::None => None,
          TyData::Struct(name) => get_struct_loc(db, semantic_data, id, name),
          data => unreachable!("bad ty: {:?}", data),
        }
      }
      _ => None,
    }
  } else if let Some(ty) = Ty::cast(tok.parent().into()) {
    let ty = syntax_data.ptrs.ty[&AstPtr::new(&ty)];
    match syntax_data.hir_root.arenas.ty[ty] {
      hir::Ty::Struct(ref name) => get_struct_loc(db, semantic_data, id, name),
      hir::Ty::Name(ref name) => get_item_loc(
        db,
        &semantic_data.import.type_defs,
        &semantic_data.env.env.type_defs,
        id,
        name,
        |item| match *item {
          hir::Item::TypeDef(ref the_name, _) => the_name == name,
          _ => false,
        },
      ),
      _ => None,
    }
  } else {
    None
  }
}

fn get_struct_loc(
  db: &Db,
  semantic_data: &SemanticData,
  id: UriId,
  name: &hir::Name,
) -> Option<Location> {
  get_item_loc(
    db,
    &semantic_data.import.structs,
    &semantic_data.env.env.structs,
    id,
    name,
    |item| match *item {
      hir::Item::Struct(ref the_name, _) => the_name == name,
      _ => false,
    },
  )
}

fn get_item_loc<IT, ET, F>(
  db: &Db,
  import_items: &FxHashMap<hir::Name, InFile<IT>>,
  env_items: &FxHashMap<hir::Name, ET>,
  id: UriId,
  name: &hir::Name,
  f: F,
) -> Option<Location>
where
  F: Fn(&hir::Item) -> bool,
{
  let def_uri_id = import_items
    .get(name)
    .and_then(|x| x.file().uri())
    .or_else(|| env_items.contains_key(name).then(|| id))?;
  let def_syntax_data = &db.syntax_data[&def_uri_id];
  let item_id = *def_syntax_data
    .hir_root
    .items
    .iter()
    .rev()
    .find(|&&id| f(&def_syntax_data.hir_root.arenas.item[id]))?;
  Some(Location {
    uri: db.uris.get(def_uri_id).clone(),
    range: def_syntax_data.positions.range(
      def_syntax_data.ptrs.item_back[item_id]
        .to_node(def_syntax_data.ast_root.syntax().clone())
        .syntax()
        .text_range(),
    ),
  })
}
