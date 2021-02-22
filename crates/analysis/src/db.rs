//! TODO could be way more incremental: only recalculate envs for the
//! transitive closure of the files that import this file, stopping if
//! updated envs are the same. use `salsa` for that?

use crate::queries::{all_diagnostics, go_to_def, hover};
use crate::types::{Diagnostic, Edit, Hover, Location, Update};
use lower::Ptrs;
use rustc_hash::FxHashMap;
use statics::{Cx, EnvWithIds, FileId, Import};
use std::hash::BuildHasherDefault;
use syntax::ast::{Root as AstRoot, Syntax as _};
use syntax::SyntaxNode;
use text_pos::{Position, PositionDb};
use topo_sort::Graph;
use uri_db::{Uri, UriDb, UriId};
use uses::UseKind;

/// A database of C0 files, which can be queried for interesting facts.
#[derive(Debug)]
pub struct Db {
  pub(crate) uris: UriDb,
  pub(crate) ordering: Vec<UriId>,
  pub(crate) syntax_data: FxHashMap<UriId, SyntaxData>,
  pub(crate) kind: DbKind,
}

impl Db {
  /// Returns a new `Db` for the given files.
  pub fn new<I>(files: I) -> Self
  where
    I: IntoIterator<Item = (Uri, String)>,
  {
    let mut uris = UriDb::default();
    let syntax_data: FxHashMap<_, _> = files
      .into_iter()
      .map(|(uri, contents)| (uris.insert(uri), get_syntax_data(contents)))
      .collect();
    get_all_semantic_data(uris, syntax_data)
  }

  /// Edit a single file.
  ///
  /// The file must already be in the `Db`.
  pub fn edit_file<I>(&mut self, uri: &Uri, edits: I)
  where
    I: IntoIterator<Item = Edit>,
  {
    let uris = std::mem::take(&mut self.uris);
    let mut syntax_data = std::mem::take(&mut self.syntax_data);
    let id = uris.get_id(uri).expect("no ID for URI in edit_file");
    let sd = syntax_data.remove(&id).unwrap();
    let mut positions = sd.positions;
    let mut contents = sd.contents;
    for edit in edits {
      match edit.range {
        None => contents = edit.text,
        Some(range) => {
          let text_range = positions.text_range(range);
          let range = std::ops::Range::<usize>::from(text_range);
          contents.replace_range(range, &edit.text);
        }
      }
      // TODO could only invalidate `positions` based on the range of the edits
      positions = PositionDb::new(&contents);
    }
    assert!(syntax_data.insert(id, get_syntax_data(contents)).is_none());
    *self = get_all_semantic_data(uris, syntax_data)
  }

  /// Update some files.
  ///
  /// The files may or may not be in the `Db`.
  pub fn update_files<I>(&mut self, updates: I)
  where
    I: IntoIterator<Item = Update>,
  {
    let mut uris = std::mem::take(&mut self.uris);
    let mut syntax_data = std::mem::take(&mut self.syntax_data);
    for update in updates {
      match update {
        Update::Create(uri, contents) => {
          let id = uris.insert(uri);
          syntax_data.insert(id, get_syntax_data(contents));
        }
        Update::Delete(uri) => {
          // can't delete from the `UriDb`.
          let id = uris.remove(&uri).expect("delete when wasn't present");
          assert!(syntax_data.remove(&id).is_some());
        }
      }
    }
    *self = get_all_semantic_data(uris, syntax_data)
  }

  /// Formats the file at the given URI.
  pub fn format(&self, uri: &Uri) -> Option<String> {
    let id = self.uris.get_id(uri)?;
    let errors = &self.syntax_data[&id].errors;
    if errors.lex.is_empty() && errors.parse.is_empty() {
      fmt::get(self.syntax_data[&id].ast_root.clone())
    } else {
      None
    }
  }

  /// Returns the parse tree of the file at the given URI.
  pub fn syntax(&self, uri: &Uri) -> Option<SyntaxNode> {
    let id = self.uris.get_id(uri)?;
    Some(self.syntax_data[&id].ast_root.syntax().clone())
  }

  /// Returns all diagnostics of every file.
  pub fn all_diagnostics(&self) -> Vec<(Uri, Vec<Diagnostic>)> {
    all_diagnostics::get(self)
  }

  /// Returns the location of the definition of the thing being pointed at.
  pub fn go_to_def(&self, uri: &Uri, pos: Position) -> Option<Location> {
    go_to_def::get(self, uri, pos)
  }

  /// Returns hover information about the thing being pointed at.
  pub fn hover(&self, uri: &Uri, pos: Position) -> Option<Hover> {
    hover::get(self, uri, pos)
  }
}

fn map_with_capacity<K, V>(cap: usize) -> FxHashMap<K, V> {
  FxHashMap::with_capacity_and_hasher(cap, BuildHasherDefault::default())
}

fn get_syntax_data(contents: String) -> SyntaxData {
  let lexed = lex::get(&contents);
  let parsed = parse::get(lexed.tokens);
  let lowered = lower::get(parsed.root.clone());
  let positions = PositionDb::new(&contents);
  // satisfy borrowck
  let lexed_uses = lexed.uses;
  let lexed_errors = lexed.errors;
  SyntaxData {
    contents,
    positions,
    ast_root: parsed.root,
    hir_root: lowered.root,
    uses: lexed_uses,
    ptrs: lowered.ptrs,
    errors: SyntaxErrors {
      lex: lexed_errors,
      parse: parsed.errors,
      lower: lowered.errors,
    },
  }
}

fn get_all_semantic_data(
  uris: UriDb,
  syntax_data: FxHashMap<UriId, SyntaxData>,
) -> Db {
  let mut uses = map_with_capacity(syntax_data.len());
  let mut uses_errors = map_with_capacity(syntax_data.len());
  for (&id, sd) in syntax_data.iter() {
    let us = uses::get(&uris, id, sd.uses.clone());
    assert!(uses.insert(id, us.uses).is_none());
    assert!(uses_errors.insert(id, us.errors).is_none());
  }
  // determine a topo ordering of the file dependencies.
  let graph: Graph<_> = syntax_data
    .keys()
    .map(|&id| {
      let neighbors = uses[&id]
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
      return Db {
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
  let mut semantic_data =
    map_with_capacity::<UriId, SemanticData>(syntax_data.len());
  for &id in ordering.iter() {
    let mut import = Import::with_main();
    for u in uses[&id].iter() {
      let (file, env) = match u.kind {
        UseKind::File(id) => (FileId::Uri(id), &semantic_data[&id].env),
        UseKind::Lib(lib) => (FileId::StdLib, std_lib.get(lib)),
      };
      statics::add_env(&mut cx, &mut import, env, file);
    }
    // we used to store this directly in the id itself, but that's a bit of a
    // pain. could go back to doing that as a micro-optimization.
    let should_define = std::path::Path::new(uris[id].path())
      .extension()
      .map_or(true, |x| x != "h0");
    let env =
      statics::get(&mut cx, &import, should_define, &syntax_data[&id].hir_root);
    semantic_data.insert(
      id,
      SemanticData {
        import,
        env,
        uses_errors: uses_errors.remove(&id).unwrap(),
        statics_errors: std::mem::take(&mut cx.errors),
      },
    );
  }
  // return.
  Db {
    uris,
    syntax_data,
    ordering,
    kind: DbKind::Done(Box::new(Done { cx, semantic_data })),
  }
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

/// Information contained in a 'done' database that had no cycle errors.
#[derive(Debug)]
pub(crate) struct Done {
  pub(crate) cx: Cx,
  pub(crate) semantic_data: FxHashMap<UriId, SemanticData>,
}

/// Syntax data for a file.
///
/// Everything in this struct is derived from `contents`.
#[derive(Debug)]
pub(crate) struct SyntaxData {
  pub(crate) contents: String,
  pub(crate) positions: PositionDb,
  pub(crate) ast_root: AstRoot,
  pub(crate) hir_root: hir::Root,
  pub(crate) ptrs: Ptrs,
  pub(crate) uses: Vec<syntax::Use>,
  pub(crate) errors: SyntaxErrors,
}

/// Syntax errors from a file.
#[derive(Debug)]
pub(crate) struct SyntaxErrors {
  pub(crate) lex: Vec<lex::Error>,
  pub(crate) parse: Vec<parse::Error>,
  pub(crate) lower: Vec<lower::PragmaError>,
}

/// Semantic data about a file.
#[derive(Debug)]
pub(crate) struct SemanticData {
  pub(crate) import: Import,
  pub(crate) env: EnvWithIds,
  pub(crate) uses_errors: Vec<uses::Error>,
  pub(crate) statics_errors: Vec<statics::Error>,
}
