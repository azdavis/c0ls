use crate::lines::Lines;
use crate::types::{Diagnostic, Hover, Location, Markdown, Position, Range};
use crate::uri::{Map, Uri};
use crate::uses::{get as get_use, Lib, Use, UseKind};
use lower::{AstPtr, Ptrs};
use rustc_hash::FxHashMap;
use statics::{
  get as get_statics, Cx, Env, FileId, FileKind, Id, Import, TyDb,
};
use std::hash::{BuildHasher, BuildHasherDefault};
use syntax::ast::{Cast as _, Expr, Root as AstRoot, Syntax as _};
use syntax::rowan::TextRange;
use topo_sort::{topological_sort, Graph};

#[derive(Debug)]
pub struct Db {
  uris: Map,
  ordering: Vec<FileId>,
  syntax_data: FxHashMap<FileId, SyntaxData>,
  kind: Kind,
}

impl Db {
  /// TODO rm header hack
  pub fn new<S>(
    files: std::collections::HashMap<Uri, String, S>,
    header: Option<(Uri, String)>,
  ) -> Self
  where
    S: BuildHasher,
  {
    // assign file IDs.
    let num_files = files.len();
    let mut uris = Map::default();
    let mut id_and_contents = map_with_capacity(num_files);
    let header_id = header.map(|(uri, contents)| {
      let id = uris.insert(uri, FileKind::Header);
      id_and_contents.insert(id, contents);
      id
    });
    for (uri, contents) in files {
      let ext = uri
        .as_path()
        .extension()
        .expect("no extension")
        .to_str()
        .expect("extension is not UTF-8");
      let kind = match ext {
        "h0" => FileKind::Header,
        _ => FileKind::Source,
      };
      let id = uris.insert(uri.clone(), kind);
      id_and_contents.insert(id, contents);
    }
    // - lex, parse, lower.
    // - process uses to resolve libraries/files.
    // - calculate line ending information.
    let mut syntax_data = map_with_capacity(num_files);
    let mut hir_roots = map_with_capacity(num_files);
    let mut uses = map_with_capacity(num_files);
    for (id, contents) in id_and_contents {
      let lexed = lex::get(&contents);
      let parsed = parse::get(lexed.tokens);
      let lowered = lower::get(parsed.root.clone());
      let mut us = Vec::with_capacity(lexed.uses.len());
      let mut uses_errors = Vec::new();
      if let Some(header_id) = header_id {
        if header_id != id {
          us.push(Use {
            kind: UseKind::File(header_id),
            range: TextRange::empty(0.into()),
          });
        }
      }
      for u in lexed.uses {
        match get_use(&uris, id, u) {
          Ok(u) => us.push(u),
          Err(e) => uses_errors.push(e),
        }
      }
      hir_roots.insert(id, lowered.root);
      uses.insert(id, us);
      syntax_data.insert(
        id,
        SyntaxData {
          lines: Lines::new(&contents),
          ast_root: parsed.root,
          ptrs: lowered.ptrs,
          errors: SyntaxErrors {
            lex: lexed.errors,
            uses: uses_errors,
            parse: parsed.errors,
            lower: lowered.errors,
          },
        },
      );
    }
    // determine a topo ordering of the file dependencies.
    let graph: Graph<_> = uses
      .iter()
      .map(|(&id, file_uses)| {
        let neighbors = file_uses
          .iter()
          .filter_map(|u| match u.kind {
            UseKind::File(id) => Some(id),
            UseKind::Lib(_) => None,
          })
          .collect();
        (id, neighbors)
      })
      .collect();
    let ordering = match topological_sort(&graph) {
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
          kind: Kind::CycleError(e.witness()),
        };
      }
    };
    // run statics in the order of the topo order, update errors.
    let (mut cx, std_lib) = std_lib::get();
    let mut semantic_data =
      map_with_capacity::<FileId, SemanticData>(num_files);
    for &id in ordering.iter() {
      let mut import = Import::with_main();
      for u in uses[&id].iter() {
        let env = match u.kind {
          UseKind::File(id) => &semantic_data[&id].env,
          UseKind::Lib(lib) => match lib {
            Lib::Args => &std_lib.args,
            Lib::Conio => &std_lib.conio,
            Lib::File => &std_lib.file,
            Lib::Img => &std_lib.img,
            Lib::Parse => &std_lib.parse,
            Lib::Rand => &std_lib.rand,
            Lib::String => &std_lib.string,
            Lib::Util => &std_lib.util,
          },
        };
        for (name, data) in env.fns.iter() {
          // TODO this should actually be the logic that checks for compatible
          // function declarations
          import.fns.insert(name.clone(), data.sig.clone());
        }
        for (name, sig) in env.structs.iter() {
          // TODO this should error if dupe
          import.structs.insert(name.clone(), sig.clone());
        }
        for (name, &ty) in env.type_defs.iter() {
          // TODO this should error if dupe
          import.type_defs.insert(name.clone(), ty);
        }
      }
      let env = get_statics(&mut cx, &import, id.kind(), &hir_roots[&id]);
      semantic_data.insert(
        id,
        SemanticData {
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
      kind: Kind::Done(Box::new(Done { cx, semantic_data })),
    }
  }

  pub fn all_diagnostics(&self) -> Vec<(&Uri, Vec<Diagnostic>)> {
    match self.kind {
      Kind::Done(ref done) => self
        .ordering
        .iter()
        .map(|&id| {
          let ds = get_diagnostics(
            &self.syntax_data[&id],
            &done.semantic_data[&id],
            &done.cx.tys,
          );
          (self.uris.get(id), ds)
        })
        .collect(),
      Kind::CycleError(witness) => self
        .ordering
        .iter()
        .map(|&id| {
          let ds =
            get_diagnostics_cycle_error(&self.syntax_data[&id], id, witness);
          (self.uris.get(id), ds)
        })
        .collect(),
    }
  }

  pub fn go_to_def(&self, _: &Uri, _: Position) -> Option<Location> {
    // TODO
    None
  }

  pub fn hover(&self, uri: &Uri, pos: Position) -> Option<Hover> {
    let done = self.kind.done()?;
    let id = self.uris.get_id(uri)?;
    let syntax_data = &self.syntax_data[&id];
    let idx = syntax_data.lines.text_size(pos);
    let tok = syntax_data
      .ast_root
      .syntax()
      .token_at_offset(idx)
      .right_biased()?;
    let mut node = tok.parent();
    let expr_node = loop {
      match Expr::cast(node.clone().into()) {
        Some(x) => break x,
        None => node = node.parent()?,
      }
    };
    let expr = *syntax_data.ptrs.expr.get(&AstPtr::new(&expr_node))?;
    let ty = *done.semantic_data[&id].env.expr_tys.get(expr)?;
    let ty = ty.display(&done.cx.tys).to_string();
    let contents = Markdown::new(format!("```c0\n{}\n```", ty));
    let range = syntax_data.lines.range(expr_node.syntax().text_range());
    Some(Hover { contents, range })
  }
}

fn get_text_range(ptrs: &Ptrs, ast_root: &AstRoot, id: Id) -> TextRange {
  let root = ast_root.syntax().clone();
  match id {
    Id::Expr(id) => ptrs.expr_back[id]
      .unwrap()
      .to_node(root)
      .syntax()
      .text_range(),
    Id::Ty(id) => ptrs.ty_back[id].to_node(root).syntax().text_range(),
    Id::Stmt(id) => ptrs.stmt_back[id].to_node(root).syntax().text_range(),
    Id::Simp(id) => ptrs.simp_back[id].to_node(root).syntax().text_range(),
    Id::Item(id) => ptrs.item_back[id].to_node(root).syntax().text_range(),
  }
}

fn get_syntax_diagnostics(
  syntax_data: &SyntaxData,
) -> impl Iterator<Item = (TextRange, String)> + '_ {
  syntax_data
    .errors
    .lex
    .iter()
    .map(|x| (x.range, x.kind.to_string()))
    .chain(
      syntax_data
        .errors
        .uses
        .iter()
        .map(|x| (x.range, x.kind.to_string())),
    )
    .chain(
      syntax_data
        .errors
        .parse
        .iter()
        .map(|x| (x.range, x.expected.to_string())),
    )
    .chain(
      syntax_data
        .errors
        .lower
        .iter()
        .map(|x| (x.range, x.to_string())),
    )
}

fn get_diagnostics(
  syntax_data: &SyntaxData,
  semantic_data: &SemanticData,
  tys: &TyDb,
) -> Vec<Diagnostic> {
  get_syntax_diagnostics(syntax_data)
    .chain(semantic_data.errors.iter().map(|x| {
      let range =
        get_text_range(&syntax_data.ptrs, &syntax_data.ast_root, x.id);
      (range, x.kind.display(tys).to_string())
    }))
    .map(|(rng, message)| Diagnostic {
      range: syntax_data.lines.range(rng),
      message,
    })
    .collect()
}

fn get_diagnostics_cycle_error(
  syntax_data: &SyntaxData,
  id: FileId,
  witness: FileId,
) -> Vec<Diagnostic> {
  let mut ret: Vec<_> = get_syntax_diagnostics(syntax_data)
    .map(|(rng, message)| Diagnostic {
      range: syntax_data.lines.range(rng),
      message,
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

fn map_with_capacity<K, V>(cap: usize) -> FxHashMap<K, V> {
  FxHashMap::with_capacity_and_hasher(cap, BuildHasherDefault::default())
}

#[derive(Debug)]
enum Kind {
  CycleError(FileId),
  Done(Box<Done>),
}

impl Kind {
  fn done(&self) -> Option<&Done> {
    match *self {
      Kind::CycleError(_) => None,
      Kind::Done(ref done) => Some(done.as_ref()),
    }
  }
}

#[derive(Debug)]
struct Done {
  cx: Cx,
  semantic_data: FxHashMap<FileId, SemanticData>,
}

/// not really 'syntax', but more in contrast to semantic info from statics.
#[derive(Debug)]
struct SyntaxData {
  lines: Lines,
  ast_root: AstRoot,
  ptrs: Ptrs,
  errors: SyntaxErrors,
}

#[derive(Debug)]
struct SyntaxErrors {
  lex: Vec<lex::Error>,
  uses: Vec<crate::uses::Error>,
  parse: Vec<parse::Error>,
  lower: Vec<lower::PragmaError>,
}

#[derive(Debug)]
struct SemanticData {
  env: Env,
  errors: Vec<statics::Error>,
}
