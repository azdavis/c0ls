use crate::db::{Db, DbKind, SemanticData, SyntaxData};
use crate::types::Diagnostic;
use lower::Ptrs;
use statics::{Id, TyDb};
use syntax::ast::{AstNode, Root};
use syntax::rowan::TextRange;
use text_pos::{Position, Range};
use uri_db::{Uri, UriId};

pub(crate) fn get(db: &Db) -> Vec<(Uri, Vec<Diagnostic>)> {
  match db.kind {
    DbKind::Done(ref done) => db
      .ordering
      .iter()
      .map(|&id| {
        let ds = get_diagnostics(
          &db.syntax_data[&id],
          &done.semantic_data[&id],
          &done.cx.tys,
        );
        (db.uris[id].clone(), ds)
      })
      .collect(),
    DbKind::CycleError(witness) => db
      .ordering
      .iter()
      .map(|&id| {
        let ds = get_diagnostics_cycle_error(&db.syntax_data[&id], id, witness);
        (db.uris[id].clone(), ds)
      })
      .collect(),
  }
}

fn get_diagnostics(
  syntax_data: &SyntaxData,
  semantic_data: &SemanticData,
  tys: &TyDb,
) -> Vec<Diagnostic> {
  get_syntax_diagnostics(syntax_data)
    .chain(
      semantic_data
        .uses_errors
        .iter()
        .map(|x| (x.range, x.kind.to_string())),
    )
    .chain(
      semantic_data
        .import_errors
        .iter()
        .map(|x| (x.range, x.kind.display(tys).to_string())),
    )
    .chain(semantic_data.statics_errors.iter().map(|x| {
      let range =
        get_text_range(&syntax_data.ptrs, &syntax_data.ast_root, x.id);
      (range, x.kind.display(tys).to_string())
    }))
    .filter_map(|(rng, message)| {
      Some(Diagnostic {
        range: syntax_data.positions.range(rng)?,
        message,
      })
    })
    .collect()
}

fn get_diagnostics_cycle_error(
  syntax_data: &SyntaxData,
  id: UriId,
  witness: UriId,
) -> Vec<Diagnostic> {
  let mut ret: Vec<_> = get_syntax_diagnostics(syntax_data)
    .filter_map(|(rng, message)| {
      Some(Diagnostic {
        range: syntax_data.positions.range(rng)?,
        message,
      })
    })
    .collect();
  if id == witness {
    let z = Position {
      line: 0,
      character: 0,
    };
    ret.push(Diagnostic {
      range: Range { start: z, end: z },
      message: "cannot have a use cycle involving this file".to_owned(),
    })
  }
  ret
}

fn get_syntax_diagnostics(
  sd: &SyntaxData,
) -> impl Iterator<Item = (TextRange, String)> + '_ {
  let lex = sd.errors.lex.iter().map(|x| (x.range, x.kind.to_string()));
  let parse = sd
    .errors
    .parse
    .iter()
    .map(|x| (x.range, x.kind.to_string()));
  let lower = sd.errors.lower.iter().map(|x| (x.range, x.to_string()));
  lex.chain(parse).chain(lower)
}

fn get_text_range(ptrs: &Ptrs, ast_root: &Root, id: Id) -> TextRange {
  let root = ast_root.syntax().clone();
  match id {
    Id::Expr(id) => ptrs.expr_back[id].to_node(&root).syntax().text_range(),
    Id::Ty(id) => ptrs.ty_back[id].to_node(&root).syntax().text_range(),
    Id::Stmt(id) => ptrs.stmt_back[id].to_node(&root).syntax().text_range(),
    Id::Simp(id) => ptrs.simp_back[id].to_node(&root).syntax().text_range(),
    Id::Item(id) => ptrs.item_back[id].to_node(&root).syntax().text_range(),
  }
}
