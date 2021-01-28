#![deny(rust_2018_idioms)]

use statics::util::{Cx, ItemDb};
use std::{env, error::Error, fs, process};
use syntax::ast::{Cast as _, Root};

fn main() {
  match run() {
    Ok(true) => eprintln!("no errors"),
    Ok(false) => process::exit(1),
    Err(e) => {
      eprintln!("{}", e);
      process::exit(2);
    }
  }
}

macro_rules! show_errors {
  ($name:expr, $errors:expr) => {
    if !$errors.is_empty() {
      eprintln!("==> {} errors ({})", $name, $errors.len());
    }
    for e in $errors.iter() {
      eprintln!("{:?}", e);
    }
  };
}

fn run() -> Result<bool, Box<dyn Error>> {
  let file = match env::args().nth(1) {
    Some(x) => x,
    None => return Err("missing first argument".into()),
  };
  let contents = fs::read_to_string(&file)?;
  let lex = lex::get(&contents);
  show_errors!("lex", lex.errors);
  let parse = parse::get(lex.tokens);
  show_errors!("parse", parse.errors);
  let mut cx = Cx::default();
  let mut items = ItemDb::default();
  let root = Root::cast(parse.tree.into()).expect("parse didn't give a root");
  statics::get(&mut cx, &mut items, root);
  show_errors!("statics", cx.errors);
  Ok(lex.errors.is_empty() && parse.errors.is_empty() && cx.errors.is_empty())
}
