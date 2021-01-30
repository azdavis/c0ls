#![deny(rust_2018_idioms)]

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::emit;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use gumdrop::Options;
use lex::Error as LexError;
use parse::{Error as ParseError, TypeDefs};
use statics::display::error_kind;
use statics::error::Error as StaticsError;
use statics::name::Name;
use statics::ty::Ty;
use statics::util::{Cx as StaticsCx, Defined, FileKind, FnData, ItemDb};
use syntax::ast::{Cast as _, Root};
use syntax::rowan::TextRange;

#[derive(Debug, Options)]
struct Config {
  #[options(help = "print this help")]
  pub help: bool,
  #[options(short = "l", help = "header file")]
  pub header: Option<String>,
  #[options(free, help = "source files")]
  pub source: Vec<String>,
}

struct Errors {
  lex: Vec<LexError>,
  parse: Vec<ParseError>,
  statics: Vec<StaticsError>,
}

type FileId = usize;

struct Cx {
  files: SimpleFiles<String, String>,
  errors: Vec<(FileId, Errors)>,
  statics: StaticsCx,
  type_defs: TypeDefs,
  items: ItemDb,
}

fn add_file(cx: &mut Cx, name: String, kind: FileKind) -> Option<()> {
  let contents = match std::fs::read_to_string(&name) {
    Ok(x) => x,
    Err(e) => {
      eprintln!("{}: {}", name, e);
      return None;
    }
  };
  let lex = lex::get(&contents);
  let parse = parse::get(lex.tokens, &mut cx.type_defs);
  let lex_errors = lex.errors;
  let handle = cx.files.add(name, contents);
  let root = Root::cast(parse.tree.into()).expect("parse didn't give a Root");
  statics::get(&mut cx.statics, &mut cx.items, kind, root);
  let errors = Errors {
    lex: lex_errors,
    parse: parse.errors,
    statics: std::mem::take(&mut cx.statics.errors),
  };
  cx.errors.push((handle, errors));
  Some(())
}

fn err(id: FileId, msg: String, range: TextRange) -> Diagnostic<FileId> {
  let label = Label::primary(id, range.start().into()..range.end().into());
  Diagnostic::error()
    .with_message(msg)
    .with_labels(vec![label])
}

fn run(conf: Config) -> Option<bool> {
  let mut cx = Cx {
    files: SimpleFiles::new(),
    errors: Vec::default(),
    statics: StaticsCx::default(),
    type_defs: TypeDefs::default(),
    items: ItemDb::default(),
  };
  cx.statics.called.insert(Name::new("main"));
  cx.items.fns.insert(
    Name::new("main"),
    FnData {
      params: vec![],
      ret_ty: Ty::Int,
      defined: Defined::NotYet,
    },
  );
  if let Some(header) = conf.header {
    add_file(&mut cx, header, FileKind::Header)?;
  }
  for source in conf.source {
    add_file(&mut cx, source, FileKind::Source)?;
  }
  let mut ok = true;
  let writer = StandardStream::stderr(ColorChoice::Auto);
  let config = codespan_reporting::term::Config::default();
  for &(id, ref es) in cx.errors.iter() {
    for e in es.lex.iter() {
      ok = false;
      let d = err(id, e.kind.to_string(), e.range);
      emit(&mut writer.lock(), &config, &cx.files, &d).unwrap();
    }
    for e in es.parse.iter() {
      ok = false;
      let d = err(id, e.expected.to_string(), e.range);
      emit(&mut writer.lock(), &config, &cx.files, &d).unwrap();
    }
    for e in es.statics.iter() {
      ok = false;
      let msg = error_kind(e.kind, &cx.statics.tys).to_string();
      let d = err(id, msg, e.range);
      emit(&mut writer.lock(), &config, &cx.files, &d).unwrap();
    }
  }
  // TODO move this into statics? at the very least we need better location
  // information.
  for name in cx.statics.called.iter() {
    let this_ok = match cx.items.fns[name].defined {
      // special case for main
      Defined::MustNot => name != "main",
      Defined::NotYet => false,
      Defined::Yes => true,
    };
    if !this_ok {
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
