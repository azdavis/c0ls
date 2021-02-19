//! TODO implement incremental updating

use crate::lines::Lines;
use crate::types::{CodeBlock, Diagnostic, Hover, Location, Position, Range};
use crate::uses::{get as get_use, UseKind};
use lower::{AstPtr, Ptrs};
use rustc_hash::FxHashMap;
use statics::{
  get as get_statics, Cx, Env, FileId, Id, Import, InFile, TyData, TyDb,
};
use std::hash::BuildHasherDefault;
use syntax::ast::{Cast as _, Expr, Root as AstRoot, Syntax as _, Ty};
use syntax::rowan::{TextRange, TokenAtOffset};
use syntax::{SyntaxKind, SyntaxNode, SyntaxToken};
use topo_sort::{topological_sort, Graph};
use uri_db::{Uri, UriDb, UriId};

#[derive(Debug)]
pub struct Db {
  uris: UriDb,
  ordering: Vec<UriId>,
  syntax_data: FxHashMap<UriId, SyntaxData>,
  kind: Kind,
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
    let mut uses = map_with_capacity(num_files);
    for (id, contents) in id_and_contents {
      let lexed = lex::get(&contents);
      let parsed = parse::get(lexed.tokens);
      let lowered = lower::get(parsed.root.clone());
      let mut us = Vec::with_capacity(lexed.uses.len());
      let mut uses_errors = Vec::new();
      for u in lexed.uses {
        match get_use(&uris, id, u) {
          Ok(u) => us.push(u),
          Err(e) => uses_errors.push(e),
        }
      }
      uses.insert(id, us);
      syntax_data.insert(
        id,
        SyntaxData {
          lines: Lines::new(&contents),
          ast_root: parsed.root,
          hir_root: lowered.root,
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
    drop(graph);
    // run statics in the order of the topo order, update errors.
    let (mut cx, std_lib) = std_lib::get();
    let mut semantic_data = map_with_capacity::<UriId, SemanticData>(num_files);
    for &id in ordering.iter() {
      let mut import = Import::with_main();
      for u in uses[&id].iter() {
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
      let env = get_statics(
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
      kind: Kind::Done(Box::new(Done { cx, semantic_data })),
    }
  }

  pub fn all_diagnostics(&self) -> Vec<(Uri, Vec<Diagnostic>)> {
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
          (self.uris.get(id).clone(), ds)
        })
        .collect(),
      Kind::CycleError(witness) => self
        .ordering
        .iter()
        .map(|&id| {
          let ds =
            get_diagnostics_cycle_error(&self.syntax_data[&id], id, witness);
          (self.uris.get(id).clone(), ds)
        })
        .collect(),
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
    let range = syntax_data.lines.range(expr_node.syntax().text_range());
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

fn get_text_range(ptrs: &Ptrs, ast_root: &AstRoot, id: Id) -> TextRange {
  let root = ast_root.syntax().clone();
  match id {
    Id::Expr(id) => ptrs.expr_back[id].to_node(root).syntax().text_range(),
    Id::Ty(id) => ptrs.ty_back[id].to_node(root).syntax().text_range(),
    Id::Stmt(id) => ptrs.stmt_back[id].to_node(root).syntax().text_range(),
    Id::Simp(id) => ptrs.simp_back[id].to_node(root).syntax().text_range(),
    Id::Item(id) => ptrs.item_back[id].to_node(root).syntax().text_range(),
  }
}

fn get_syntax_diagnostics(
  sd: &SyntaxData,
) -> impl Iterator<Item = (TextRange, String)> + '_ {
  let lex = sd.errors.lex.iter().map(|x| (x.range, x.kind.to_string()));
  let uses = sd.errors.uses.iter().map(|x| (x.range, x.kind.to_string()));
  let parse = sd
    .errors
    .parse
    .iter()
    .map(|x| (x.range, x.expected.to_string()));
  let lower = sd.errors.lower.iter().map(|x| (x.range, x.to_string()));
  lex.chain(uses).chain(parse).chain(lower)
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
  id: UriId,
  witness: UriId,
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

fn get_token(syntax_data: &SyntaxData, pos: Position) -> Option<SyntaxToken> {
  let idx = syntax_data.lines.text_size(pos);
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
    range: def_syntax_data.lines.range(get_text_range(
      &def_syntax_data.ptrs,
      &def_syntax_data.ast_root,
      item_id.into(),
    )),
  })
}

#[derive(Debug)]
enum Kind {
  CycleError(UriId),
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
  semantic_data: FxHashMap<UriId, SemanticData>,
}

/// not really 'syntax', but more in contrast to semantic info from statics.
#[derive(Debug)]
struct SyntaxData {
  lines: Lines,
  ast_root: AstRoot,
  hir_root: hir::Root,
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
  import: Import,
  env: Env,
  errors: Vec<statics::Error>,
}
