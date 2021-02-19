//! TODO implement incremental updating

use crate::queries::{all_diagnostics, go_to_def, hover};
use crate::types::{Diagnostic, Hover, Location};
use crate::util::map_with_capacity;
use lower::Ptrs;
use rustc_hash::FxHashMap;
use statics::{Cx, EnvWithIds, FileId, Import};
use syntax::ast::{Root as AstRoot, Syntax as _};
use syntax::SyntaxNode;
use text_pos::{Position, PositionDb};
use topo_sort::Graph;
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
    for uri in files.keys() {
      uris.insert(uri.clone());
    }
    // get syntax data for each file.
    let syntax_data: FxHashMap<_, _> = files
      .into_iter()
      .map(|(uri, contents)| {
        let id = uris.get_id(&uri).unwrap();
        (id, get_syntax_data(&uris, id, &contents))
      })
      .collect();
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
        let (file, env) = match u.kind {
          UseKind::File(id) => (FileId::Uri(id), &semantic_data[&id].env),
          UseKind::Lib(lib) => (FileId::StdLib, std_lib.get(lib)),
        };
        statics::add_env(&mut cx, &mut import, env, file);
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

  pub fn all_diagnostics(&self) -> Vec<(Uri, Vec<Diagnostic>)> {
    all_diagnostics::get(self)
  }

  pub fn go_to_def(&self, uri: &Uri, pos: Position) -> Option<Location> {
    go_to_def::get(self, uri, pos)
  }

  pub fn hover(&self, uri: &Uri, pos: Position) -> Option<Hover> {
    hover::get(self, uri, pos)
  }
}

fn get_syntax_data(uris: &UriDb, id: UriId, contents: &str) -> SyntaxData {
  let lexed = lex::get(contents);
  let parsed = parse::get(lexed.tokens);
  let lowered = lower::get(parsed.root.clone());
  let uses = uses::get(&uris, id, lexed.uses);
  SyntaxData {
    positions: PositionDb::new(contents),
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
  pub(crate) env: EnvWithIds,
  pub(crate) errors: Vec<statics::Error>,
}
