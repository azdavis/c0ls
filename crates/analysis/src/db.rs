//! TODO implement incremental updating

use crate::queries::all_diagnostics;
use crate::types::{CodeBlock, Diagnostic, Hover, Location};
use lower::{AstPtr, Ptrs};
use rustc_hash::FxHashMap;
use statics::{Cx, Env, FileId, Import, InFile, TyData};
use std::hash::BuildHasherDefault;
use syntax::ast::{Cast as _, Expr, Root as AstRoot, Syntax as _, Ty};
use syntax::rowan::TokenAtOffset;
use syntax::{SyntaxKind, SyntaxNode, SyntaxToken};
use text_pos::{Position, PositionDb};
use topo_sort::{self, Graph};
use uri_db::{Uri, UriDb, UriId};
use uses::{Use, UseKind};

#[derive(Debug)]
pub struct Db {
  pub(crate) uris: UriDb,
  pub(crate) ordering: Vec<UriId>,
  pub(crate) syntax_data: FxHashMap<UriId, SyntaxData>,
  pub(crate) kind: DbKind,
}

impl Db {
  pub fn new(files: FxHashMap<Uri, String>) -> Self {
    // assign file IDs.
    let num_files = files.len();
    let mut uris = UriDb::default();
    let mut id_and_contents = map_with_capacity(num_files);
    for (uri, contents) in files {
      let id = uris.insert(uri.clone());
      id_and_contents.insert(id, contents);
    }
    // - lex, parse, lower.
    // - process uses to resolve libraries/files.
    // - calculate line ending information.
    let mut syntax_data = map_with_capacity(num_files);
    for (id, contents) in id_and_contents {
      syntax_data.insert(id, get_syntax_data(&uris, id, &contents));
    }
    // determine a topo ordering of the file dependencies.
    let graph: Graph<_> = syntax_data
      .iter()
      .map(|(&id, sd)| {
        let neighbors = sd
          .uses
          .iter()
          .filter_map(|u| match u.kind {
            UseKind::File(id) => Some(id),
            UseKind::Lib(_) => None,
          })
          .collect();
        (id, neighbors)
      })
      .collect();
    let ordering = match topo_sort::get(&graph) {
      Ok(x) => x,
      Err(e) => {
        // give up on further processing. conjure up a stable but arbitrary
        // ordering.
        let mut ordering: Vec<_> = uris.iter().collect();
        ordering.sort_unstable();
        return Self {
          uris,
          syntax_data,
          ordering,
          kind: DbKind::CycleError(e.witness()),
        };
      }
    };
    drop(graph);
    // run statics in the order of the topo order, update errors.
    let (mut cx, std_lib) = std_lib::get();
    let mut semantic_data = map_with_capacity::<UriId, SemanticData>(num_files);
    for &id in ordering.iter() {
      let mut import = Import::with_main();
      for u in syntax_data[&id].uses.iter() {
        let (file_id, env) = match u.kind {
          UseKind::File(id) => (FileId::Uri(id), &semantic_data[&id].env),
          UseKind::Lib(lib) => (FileId::StdLib, std_lib.get(lib)),
        };
        for (name, data) in env.fns.iter() {
          // TODO this should actually be the logic that checks for compatible
          // function declarations
          let val = file_id.wrap(data.sig.clone());
          import.fns.insert(name.clone(), val);
        }
        for (name, sig) in env.structs.iter() {
          // TODO this should error if dupe
          let val = file_id.wrap(sig.clone());
          import.structs.insert(name.clone(), val);
        }
        for (name, &ty) in env.type_defs.iter() {
          // TODO this should error if dupe
          import.type_defs.insert(name.clone(), file_id.wrap(ty));
        }
      }
      let env = statics::get(
        &mut cx,
        &import,
        FileId::Uri(id),
        &syntax_data[&id].hir_root,
      );
      semantic_data.insert(
        id,
        SemanticData {
          import,
          env,
          errors: std::mem::take(&mut cx.errors),
        },
      );
    }
    // return.
    Self {
      uris,
      syntax_data,
      ordering,
      kind: DbKind::Done(Box::new(Done { cx, semantic_data })),
    }
  }

  pub fn all_diagnostics(&self) -> Vec<(Uri, Vec<Diagnostic>)> {
    all_diagnostics::get(self)
  }

  pub fn format(&self, uri: &Uri) -> Option<String> {
    let id = self.uris.get_id(uri)?;
    let errors = &self.syntax_data[&id].errors;
    if errors.lex.is_empty() && errors.parse.is_empty() {
      fmt::get(self.syntax_data[&id].ast_root.clone())
    } else {
      None
    }
  }

  pub fn syntax(&self, uri: &Uri) -> Option<SyntaxNode> {
    let id = self.uris.get_id(uri)?;
    Some(self.syntax_data[&id].ast_root.syntax().clone())
  }

  pub fn go_to_def(&self, uri: &Uri, pos: Position) -> Option<Location> {
    let done = self.kind.done()?;
    let id = self.uris.get_id(uri)?;
    let syntax_data = &self.syntax_data[&id];
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
            self,
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
          return get_struct_loc(self, semantic_data, id, name);
        }
        _ => return None,
      }
    }
    if let Some(ty) = Ty::cast(node.into()) {
      let ty = syntax_data.ptrs.ty[&AstPtr::new(&ty)];
      match syntax_data.hir_root.arenas.ty[ty] {
        hir::Ty::Struct(ref name) => {
          return get_struct_loc(self, semantic_data, id, name)
        }
        hir::Ty::Name(ref name) => {
          return get_item_loc(
            self,
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

  pub fn hover(&self, uri: &Uri, pos: Position) -> Option<Hover> {
    let done = self.kind.done()?;
    let id = self.uris.get_id(uri)?;
    let syntax_data = &self.syntax_data[&id];
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
        .or_else(|| semantic_data.env.fns.get(name).map(|x| &x.sig))?
        .display(name, &done.cx.tys)
        .to_string(),
      _ => semantic_data
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
}

fn get_syntax_data(uris: &UriDb, id: UriId, contents: &str) -> SyntaxData {
  let lexed = lex::get(&contents);
  let parsed = parse::get(lexed.tokens);
  let lowered = lower::get(parsed.root.clone());
  let uses = uses::get(&uris, id, lexed.uses);
  SyntaxData {
    positions: PositionDb::new(&contents),
    ast_root: parsed.root,
    hir_root: lowered.root,
    ptrs: lowered.ptrs,
    uses: uses.uses,
    errors: SyntaxErrors {
      lex: lexed.errors,
      uses: uses.errors,
      parse: parsed.errors,
      lower: lowered.errors,
    },
  }
}

fn map_with_capacity<K, V>(cap: usize) -> FxHashMap<K, V> {
  FxHashMap::with_capacity_and_hasher(cap, BuildHasherDefault::default())
}

fn get_token(syntax_data: &SyntaxData, pos: Position) -> Option<SyntaxToken> {
  let idx = syntax_data.positions.text_size(pos);
  let ret = match syntax_data.ast_root.syntax().token_at_offset(idx) {
    TokenAtOffset::None => return None,
    TokenAtOffset::Single(t) => t,
    TokenAtOffset::Between(t1, t2) => {
      // right biased when eq
      if priority(t1.kind()) > priority(t2.kind()) {
        t1
      } else {
        t2
      }
    }
  };
  Some(ret)
}

// heuristic for how much we should care about some token
fn priority(kind: SyntaxKind) -> u8 {
  match kind {
    SyntaxKind::Ident => 4,
    SyntaxKind::DecLit
    | SyntaxKind::HexLit
    | SyntaxKind::StringLit
    | SyntaxKind::CharLit => 3,
    SyntaxKind::IntKw
    | SyntaxKind::BoolKw
    | SyntaxKind::StringKw
    | SyntaxKind::CharKw
    | SyntaxKind::VoidKw => 2,
    SyntaxKind::Whitespace
    | SyntaxKind::LineComment
    | SyntaxKind::BlockComment
    | SyntaxKind::Invalid => 0,
    _ => 1,
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

#[derive(Debug)]
pub(crate) enum DbKind {
  CycleError(UriId),
  Done(Box<Done>),
}

impl DbKind {
  pub(crate) fn done(&self) -> Option<&Done> {
    match *self {
      DbKind::CycleError(_) => None,
      DbKind::Done(ref done) => Some(done.as_ref()),
    }
  }
}

#[derive(Debug)]
pub(crate) struct Done {
  pub(crate) cx: Cx,
  pub(crate) semantic_data: FxHashMap<UriId, SemanticData>,
}

/// not really 'syntax', but more in contrast to semantic info from statics.
#[derive(Debug)]
pub(crate) struct SyntaxData {
  pub(crate) positions: PositionDb,
  pub(crate) ast_root: AstRoot,
  pub(crate) hir_root: hir::Root,
  pub(crate) ptrs: Ptrs,
  pub(crate) uses: Vec<Use>,
  pub(crate) errors: SyntaxErrors,
}

#[derive(Debug)]
pub(crate) struct SyntaxErrors {
  pub(crate) lex: Vec<lex::Error>,
  pub(crate) uses: Vec<uses::Error>,
  pub(crate) parse: Vec<parse::Error>,
  pub(crate) lower: Vec<lower::PragmaError>,
}

#[derive(Debug)]
pub(crate) struct SemanticData {
  pub(crate) import: Import,
  pub(crate) env: Env,
  pub(crate) errors: Vec<statics::Error>,
}
