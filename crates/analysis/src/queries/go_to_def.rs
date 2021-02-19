use crate::db::{get_token, Db, SemanticData};
use crate::types::Location;
use lower::AstPtr;
use rustc_hash::FxHashMap;
use statics::{InFile, TyData};
use syntax::ast::{Cast as _, Expr, Syntax as _, Ty};
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
  let node = tok.parent();
  let semantic_data = &done.semantic_data[&id];
  if let Some(expr) = Expr::cast(node.clone().into()) {
    let expr = syntax_data.ptrs.expr[&AstPtr::new(&expr)];
    match syntax_data.hir_root.arenas.expr[expr] {
      hir::Expr::Call(ref name, _) => {
        return get_item_loc(
          db,
          &semantic_data.import.fns,
          &semantic_data.env.fns,
          id,
          name,
          |item| match *item {
            hir::Item::Fn(ref the_name, _, _, _) => the_name == name,
            _ => false,
          },
        );
      }
      hir::Expr::FieldGet(expr, _) => {
        let name = match done.cx.tys.get(semantic_data.env.expr_tys[expr]) {
          TyData::None => return None,
          TyData::Struct(name) => name,
          data => unreachable!("bad ty: {:?}", data),
        };
        return get_struct_loc(db, semantic_data, id, name);
      }
      _ => return None,
    }
  }
  if let Some(ty) = Ty::cast(node.into()) {
    let ty = syntax_data.ptrs.ty[&AstPtr::new(&ty)];
    match syntax_data.hir_root.arenas.ty[ty] {
      hir::Ty::Struct(ref name) => {
        return get_struct_loc(db, semantic_data, id, name)
      }
      hir::Ty::Name(ref name) => {
        return get_item_loc(
          db,
          &semantic_data.import.type_defs,
          &semantic_data.env.type_defs,
          id,
          name,
          |item| match *item {
            hir::Item::TypeDef(ref the_name, _) => the_name == name,
            _ => false,
          },
        );
      }
      _ => return None,
    }
  }
  None
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
    &semantic_data.env.structs,
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
