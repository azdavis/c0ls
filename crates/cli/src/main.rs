#![deny(rust_2018_idioms)]

use gumdrop::Options;
use lex::LexError;
use parse::{Parse, TypeDefs};
use statics::name::Name;
use statics::ty::Ty;
use statics::util::{Cx, FileKind, FnData, ItemDb};
use syntax::ast::{Cast as _, Root};
use syntax::SyntaxNode;

#[derive(Debug, Options)]
pub struct Config {
  #[options(help = "print this help")]
  pub help: bool,
  #[options(short = "l", help = "header file")]
  pub header: Option<String>,
  #[options(free, help = "source files")]
  pub source: Vec<String>,
}

macro_rules! show_errors {
  ($pass:expr, $name:expr, $errors:expr) => {
    if !$errors.is_empty() {
      eprintln!("==> {} {} errors ({})", $pass, $name, $errors.len());
    }
    for e in $errors.iter() {
      eprintln!("{:?}", e);
    }
  };
}

fn read_file(name: &str) -> Option<String> {
  match std::fs::read_to_string(name) {
    Ok(x) => Some(x),
    Err(e) => {
      eprintln!("{}: {}", name, e);
      None
    }
  }
}

fn parse_one(name: &str, tds: &mut TypeDefs) -> Option<(Vec<LexError>, Parse)> {
  let s = read_file(&name)?;
  let lex = lex::get(&s);
  show_errors!("lex", name, lex.errors);
  let parse = parse::get(lex.tokens, tds);
  show_errors!("parse", name, parse.errors);
  Some((lex.errors, parse))
}

fn root(node: SyntaxNode) -> Root {
  Root::cast(node.into()).expect("parse didn't give a Root")
}

fn run(conf: Config) -> Option<bool> {
  let mut cx = Cx::default();
  cx.called.insert(Name::new("main"));
  let mut items = ItemDb::default();
  let mut tds = TypeDefs::default();
  items.fns.insert(
    Name::new("main"),
    FnData {
      params: vec![],
      ret_ty: Ty::Int,
      defined: false,
    },
  );
  let mut ok = true;
  if let Some(header) = conf.header {
    let (header_lex_errors, header_parse) = parse_one(&header, &mut tds)?;
    let header_root = root(header_parse.tree);
    statics::get(&mut cx, &mut items, FileKind::Header, header_root);
    show_errors!("statics", header, cx.errors);
    ok = ok
      && header_lex_errors.is_empty()
      && header_parse.errors.is_empty()
      && cx.errors.is_empty();
    cx.errors.clear();
  }
  for source in conf.source {
    let (source_lex_errors, source_parse) = parse_one(&source, &mut tds)?;
    let source_root = root(source_parse.tree);
    statics::get(&mut cx, &mut items, FileKind::Source, source_root);
    show_errors!("statics", source, cx.errors);
    ok = ok
      && source_lex_errors.is_empty()
      && source_parse.errors.is_empty()
      && cx.errors.is_empty();
    cx.errors.clear();
  }
  for name in cx.called.iter() {
    if !items.fns[name].defined {
      ok = false;
      eprintln!("`{}` called but not defined", name);
    }
  }
  Some(ok)
}

const BIG_STACK_SIZE: usize = 180 * 1024 * 1024;

fn main() {
  let conf = Config::parse_args_default_or_exit();
  let ec = match std::thread::Builder::new()
    .name("run".to_owned())
    .stack_size(BIG_STACK_SIZE)
    .spawn(|| run(conf))
    .expect("couldn't spawn")
    .join()
  {
    Ok(Some(true)) => {
      eprintln!("no errors");
      0
    }
    Ok(Some(false)) => 1,
    Ok(None) => 2,
    Err(_) => 3,
  };
  std::process::exit(ec)
}
