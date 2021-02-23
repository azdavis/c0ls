use crate::db::Db;
use crate::types::Location;
use crate::util::get_token;
use lower::AstPtr;
use rustc_hash::FxHashMap;
use statics::{ItemData, TyData};
use syntax::ast::{
  Cast as _, Expr, Param, Simp, SimpOpt, SimpStmt, Syntax as _, Ty,
};
use syntax::{SyntaxKind, SyntaxToken};
use text_pos::Position;
use uri_db::Uri;

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
        let mut node = tok.parent().parent()?;
        loop {
          let declares = SimpStmt::cast(node.clone().into())
            .and_then(|x| simp_def(x.simp()?))
            .or_else(|| {
              SimpOpt::cast(node.clone().into())
                .and_then(|x| simp_def(x.simp()?))
            })
            .or_else(|| {
              Param::cast(node.clone().into()).and_then(|x| x.ident())
            })
            .map_or(false, |tok| name == tok.text());
          if declares {
            break;
          }
          // go up and to the left. not quite correct in the case of a decl in
          // the step of a for loop, but that's an error anyway.
          node = node.prev_sibling().or_else(|| node.parent())?;
        }
        Some(Location {
          uri: uri.clone(),
          range: syntax_data.positions.range(node.text_range()),
        })
      }
      hir::Expr::Call(ref name, _) => {
        get_item_loc(db, &semantic_data.env.fns, name)
      }
      hir::Expr::FieldGet(expr, _) => {
        match done.cx.tys.get(semantic_data.env.expr_tys[expr]) {
          TyData::None => None,
          TyData::Struct(name) => {
            get_item_loc(db, &semantic_data.env.structs, name)
          }
          data => unreachable!("bad ty: {:?}", data),
        }
      }
      _ => None,
    }
  } else if let Some(ty) = Ty::cast(tok.parent().into()) {
    let ty = syntax_data.ptrs.ty[&AstPtr::new(&ty)];
    match syntax_data.hir_root.arenas.ty[ty] {
      hir::Ty::Struct(ref name) => {
        get_item_loc(db, &semantic_data.env.structs, name)
      }
      hir::Ty::Name(ref name) => {
        get_item_loc(db, &semantic_data.env.type_defs, name)
      }
      _ => None,
    }
  } else {
    None
  }
}

fn simp_def(simp: Simp) -> Option<SyntaxToken> {
  match simp {
    Simp::DeclSimp(simp) => simp.ident(),
    Simp::AmbiguousSimp(simp) => simp.rhs(),
    _ => None,
  }
}

fn get_item_loc<T>(
  db: &Db,
  items: &FxHashMap<hir::Name, ItemData<T>>,
  name: &hir::Name,
) -> Option<Location> {
  let (uri, item) = items.get(name)?.id()?;
  let def_syntax_data = &db.syntax_data[&uri];
  Some(Location {
    uri: db.uris[uri].clone(),
    range: def_syntax_data.positions.range(
      def_syntax_data.ptrs.item_back[item]
        .to_node(def_syntax_data.ast_root.syntax().clone())
        .syntax()
        .text_range(),
    ),
  })
}
