//! FIXME could be way more incremental: only recalculate envs for the
//! transitive closure of the files that import this file, stopping if updated
//! envs are the same. use `salsa` for that?

use crate::queries::{all_diagnostics, go_to_def, hover};
use crate::types::{Diagnostic, Edit, Hover, Location, Update};
use lower::Ptrs;
use rustc_hash::FxHashMap;
use statics::{Cx, Env, FileId};
use std::hash::BuildHasherDefault;
use syntax::ast::{AstNode, Root as AstRoot};
use syntax::rowan::TextRange;
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
    let id = match self.uris.get_id(uri) {
      Some(x) => x,
      None => return,
    };
    let uris = std::mem::take(&mut self.uris);
    let mut syntax_data = std::mem::take(&mut self.syntax_data);
    let sd = syntax_data.remove(&id).expect("got URI but no syntax data");
    let mut positions = Some(sd.positions);
    let mut contents = sd.contents;
    for edit in edits {
      match edit.range {
        None => contents = edit.text,
        Some(range) => {
          // FIXME could only invalidate `positions` based on the range of the
          // edits
          let text_range = positions
            .unwrap_or_else(|| PositionDb::new(&contents))
            .text_range(range);
          if let Some(text_range) = text_range {
            let range = std::ops::Range::<usize>::from(text_range);
            contents.replace_range(range, &edit.text);
          }
        }
      }
      positions = None;
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
        Update::Delete(uri) => match uris.remove(&uri) {
          Some(id) => {
            syntax_data.remove(&id);
          }
          None => continue,
        },
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
  let parsed = parse::get(&lexed.tokens);
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
    let mut import = Env::with_main();
    let mut import_errors = Vec::new();
    for u in uses[&id].iter() {
      let env = match u.kind {
        UseKind::File(id) => &semantic_data[&id].env,
        UseKind::Lib(lib) => std_lib.get(lib),
      };
      let mut errors = Vec::new();
      statics::add_env(&mut cx, &mut errors, &mut import, env);
      import_errors.extend(errors.into_iter().map(|kind| ImportError {
        range: u.range,
        kind,
      }));
    }
    // we used to store this directly in the id itself, but that's a bit of a
    // pain. could go back to doing that as a micro-optimization.
    let is_header = std::path::Path::new(uris[id].path())
      .extension()
      .map_or(true, |x| x == "h0");
    let file = if is_header {
      FileId::Header(id)
    } else {
      FileId::Source(id)
    };
    let env = statics::get(&mut cx, import, file, &syntax_data[&id].hir_root);
    semantic_data.insert(
      id,
      SemanticData {
        env,
        uses_errors: uses_errors.remove(&id).expect("missing uses errors"),
        import_errors,
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
  pub(crate) env: Env,
  pub(crate) uses_errors: Vec<uses::Error>,
  pub(crate) import_errors: Vec<ImportError>,
  pub(crate) statics_errors: Vec<statics::Error>,
}

#[derive(Debug)]
pub(crate) struct ImportError {
  pub range: TextRange,
  pub kind: statics::ErrorKind,
}
