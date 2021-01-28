#![deny(rust_2018_idioms)]

use statics::util::{Cx, ItemDb};
use std::{env, fs};
use syntax::ast::{Cast as _, Root};

fn main() {
  let file = env::args().nth(1).unwrap();
  let contents = fs::read_to_string(&file).unwrap();
  let lex = lex::get(&contents);
  eprintln!("==> lex errors ({})", lex.errors.len());
  for e in lex.errors {
    eprintln!("{:?}", e);
  }
  let parse = parse::get(lex.tokens);
  eprintln!("==> parse errors ({})", parse.errors.len());
  for e in parse.errors {
    eprintln!("{:?}", e);
  }
  let mut cx = Cx::default();
  let mut items = ItemDb::default();
  let root = Root::cast(parse.tree).unwrap();
  statics::get(&mut cx, &mut items, root);
  eprintln!("==> statics errors ({})", cx.errors.len());
  for e in cx.errors {
    eprintln!("{:?}", e);
  }
}
