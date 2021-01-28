#![deny(rust_2018_idioms)]

use statics::util::{Cx, ItemDb};
use std::{env, error::Error, fs, process};
use syntax::ast::{Cast as _, Root};

fn main() {
  match run() {
    Ok(true) => {}
    Ok(false) => process::exit(1),
    Err(e) => {
      eprintln!("{}", e);
      process::exit(2);
    }
  }
}

fn run() -> Result<bool, Box<dyn Error>> {
  let file = match env::args().nth(1) {
    Some(x) => x,
    None => return Err("missing first argument".into()),
  };
  let contents = fs::read_to_string(&file)?;
  let lex = lex::get(&contents);
  eprintln!("==> lex errors ({})", lex.errors.len());
  for e in lex.errors.iter() {
    eprintln!("{:?}", e);
  }
  let parse = parse::get(lex.tokens);
  eprintln!("==> parse errors ({})", parse.errors.len());
  for e in parse.errors.iter() {
    eprintln!("{:?}", e);
  }
  let mut cx = Cx::default();
  let mut items = ItemDb::default();
  let root = Root::cast(parse.tree).expect("parse didn't give a root");
  statics::get(&mut cx, &mut items, root);
  eprintln!("==> statics errors ({})", cx.errors.len());
  for e in cx.errors.iter() {
    eprintln!("{:?}", e);
  }
  Ok(lex.errors.is_empty() && parse.errors.is_empty() && cx.errors.is_empty())
}
