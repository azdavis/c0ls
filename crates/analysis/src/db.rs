use crate::lines::Lines;
use crate::types::{Diagnostic, Hover, Location, Markdown, Position, Range};
use crate::uri::{Map, Uri};
use crate::uses::{get as get_use, Use, UseKind};
use lower::{AstPtr, Ptrs};
use rustc_hash::FxHashMap;
use statics_neue::{
  get as get_statics, Cx, Env, FileId, FileKind, Id, Import, TyDb,
};
use std::hash::{BuildHasher, BuildHasherDefault};
use syntax::ast::{Cast as _, Expr, Root as AstRoot, Syntax as _};
use syntax::rowan::TextRange;
use topo_sort::{topological_sort, Graph};

pub struct Db {
  uris: Map,
  lines: FxHashMap<FileId, Lines>,
  errors: FxHashMap<FileId, Errors>,
  ordering: Vec<FileId>,
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
    // - assign file IDs
    // - lex
    // - calculate line ending information
    let mut uris = Map::default();
    let mut lexes = map_with_capacity(files.len());
    let mut lines = map_with_capacity(files.len());
    for (uri, contents) in files
      .iter()
      .chain(header.as_ref().map(|&(ref u, ref c)| (u, c)))
    {
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
      lexes.insert(id, lex::get(contents));
      lines.insert(id, Lines::new(contents));
    }
    let header_id = header.as_ref().map(|&(ref u, _)| uris.get_id(u).unwrap());
    // - separate each lex into (tokens, uses, errors)
    // - process uses to resolve libraries/files
    // - put tokens, processed uses, lex errors + resolution errors each into
    //   separate maps
    let mut tokens = map_with_capacity(files.len());
    let mut uses = map_with_capacity(files.len());
    let mut errors = map_with_capacity(files.len());
    for (id, lex) in lexes {
      let mut us = Vec::with_capacity(lex.uses.len());
      let mut uses_errors = Vec::new();
      if let Some(header_id) = header_id {
        if header_id != id {
          us.push(Use {
            kind: UseKind::File(header_id),
            range: TextRange::empty(0.into()),
          });
        }
      }
      for u in lex.uses {
        match get_use(&uris, id, u) {
          Ok(u) => us.push(u),
          Err(e) => uses_errors.push(e),
        }
      }
      let es = Errors {
        lex: lex.errors,
        uses: uses_errors,
        parse: vec![],
        lower: vec![],
        statics: vec![],
      };
      tokens.insert(id, lex.tokens);
      errors.insert(id, es);
      uses.insert(id, us);
    }
    // determine a topo ordering of the file dependencies
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
        // ordering
        let mut ordering: Vec<_> = uris.iter().collect();
        ordering.sort_unstable();
        return Self {
          uris,
          errors,
          lines,
          ordering,
          kind: Kind::CycleError(e.witness()),
        };
      }
    };
    // parse + lower in the order of the topo order, update errors
    let mut type_defs = parse::TypeDefs::default();
    let mut ast_roots = map_with_capacity(files.len());
    let mut ptrs = map_with_capacity(files.len());
    let mut hir_roots = map_with_capacity(files.len());
    for &id in ordering.iter() {
      let ts = tokens.remove(&id).unwrap();
      let p = parse::get(ts, &mut type_defs);
      ast_roots.insert(id, AstRoot::cast(p.tree.clone().into()).unwrap());
      let lowered = lower::get(AstRoot::cast(p.tree.into()).unwrap());
      ptrs.insert(id, lowered.ptrs);
      hir_roots.insert(id, lowered.root);
      let es = errors.get_mut(&id).unwrap();
      es.parse = p.errors;
      es.lower = lowered.errors;
    }
    // run statics in the order of the topo order, update errors. TODO this
    // should detect duplicate/incompatible declarations across imports
    let mut cx = Cx::default();
    let mut envs = map_with_capacity::<FileId, Env>(files.len());
    for &id in ordering.iter() {
      let mut import = Import::default();
      for &id in graph[&id].iter() {
        let env = &envs[&id];
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
      let env = get_statics(&mut cx, &import, id, &hir_roots[&id]);
      envs.insert(id, env);
      let es = errors.get_mut(&id).unwrap();
      es.statics = std::mem::take(&mut cx.errors);
    }
    // return
    Self {
      uris,
      lines,
      errors,
      ordering,
      kind: Kind::Done(Box::new(Done {
        ast_roots,
        ptrs,
        cx,
        envs,
      })),
    }
  }

  pub fn all_diagnostics(&self) -> Vec<(&Uri, Vec<Diagnostic>)> {
    match self.kind {
      Kind::Done(ref done) => self
        .ordering
        .iter()
        .map(|&id| {
          let ds = get_diagnostics(
            &self.errors[&id],
            &self.lines[&id],
            &done.ptrs[&id],
            &done.ast_roots[&id],
            &done.cx.tys,
          );
          (self.uris.get(id), ds)
        })
        .collect(),
      Kind::CycleError(witness) => self
        .ordering
        .iter()
        .map(|&id| {
          let ds = get_diagnostics_cycle_error(
            &self.errors[&id],
            &self.lines[&id],
            id,
            witness,
          );
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
    let lines = &self.lines[&id];
    let idx = lines.text_size(pos);
    let tok = done.ast_roots[&id]
      .syntax()
      .token_at_offset(idx)
      .left_biased()?;
    let node = Expr::cast(tok.parent().into())?;
    let expr = *done.ptrs[&id].expr.get(&AstPtr::new(&node))?;
    let ty = *done.envs[&id].expr_tys.get(expr)?;
    let ty = ty.display(&done.cx.tys).to_string();
    let contents = Markdown::new(format!("```c0\n{}\n```", ty));
    let range = lines.range(node.syntax().text_range());
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

fn get_diagnostics(
  errors: &Errors,
  lines: &Lines,
  ptrs: &Ptrs,
  ast_root: &AstRoot,
  tys: &TyDb,
) -> Vec<Diagnostic> {
  errors
    .lex
    .iter()
    .map(|x| (x.range, x.kind.to_string()))
    .chain(errors.uses.iter().map(|x| (x.range, x.kind.to_string())))
    .chain(
      errors
        .parse
        .iter()
        .map(|x| (x.range, x.expected.to_string())),
    )
    .chain(errors.lower.iter().map(|x| (x.range, x.to_string())))
    .chain(errors.statics.iter().map(|x| {
      let range = get_text_range(ptrs, ast_root, x.id);
      (range, x.kind.display(tys).to_string())
    }))
    .map(|(rng, message)| Diagnostic {
      range: lines.range(rng),
      message,
    })
    .collect()
}

fn get_diagnostics_cycle_error(
  errors: &Errors,
  lines: &Lines,
  id: FileId,
  witness: FileId,
) -> Vec<Diagnostic> {
  assert!(errors.parse.is_empty());
  assert!(errors.lower.is_empty());
  assert!(errors.statics.is_empty());
  let mut ret: Vec<_> = errors
    .lex
    .iter()
    .map(|x| (x.range, x.kind.to_string()))
    .chain(errors.uses.iter().map(|x| (x.range, x.kind.to_string())))
    .map(|(rng, message)| Diagnostic {
      range: lines.range(rng),
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

struct Done {
  ast_roots: FxHashMap<FileId, AstRoot>,
  ptrs: FxHashMap<FileId, Ptrs>,
  cx: Cx,
  envs: FxHashMap<FileId, Env>,
}

struct Errors {
  lex: Vec<lex::Error>,
  uses: Vec<crate::uses::Error>,
  parse: Vec<parse::Error>,
  lower: Vec<lower::PragmaError>,
  statics: Vec<statics_neue::Error>,
}
