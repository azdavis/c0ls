//! The C0 standard library.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use statics::{Cx, Env, FileKind::Header, Import};

#[derive(Debug)]
pub struct StdLib {
  pub args: Env,
  pub conio: Env,
  pub file: Env,
  pub img: Env,
  pub parse: Env,
  pub rand: Env,
  pub string: Env,
  pub util: Env,
}

/// the Cx will always have no errors. the TyDb inside the Cx allows getting
/// information about the returned Envs inside the StdLib.
pub fn get() -> (Cx, StdLib) {
  let mut cx = Cx::default();
  let std_lib = StdLib {
    args: get_one(&mut cx, include_str!("data/args.h0")),
    conio: get_one(&mut cx, include_str!("data/conio.h0")),
    file: get_one(&mut cx, include_str!("data/file.h0")),
    img: get_one(&mut cx, include_str!("data/img.h0")),
    parse: get_one(&mut cx, include_str!("data/parse.h0")),
    rand: get_one(&mut cx, include_str!("data/rand.h0")),
    string: get_one(&mut cx, include_str!("data/string.h0")),
    util: get_one(&mut cx, include_str!("data/util.h0")),
  };
  (cx, std_lib)
}

fn get_one(cx: &mut Cx, contents: &str) -> Env {
  let lexed = lex::get(contents);
  let parsed = parse::get(lexed.tokens);
  let lowered = lower::get(parsed.root);
  let ret = statics::get(cx, &Import::default(), Header, &lowered.root);
  assert!(lexed.errors.is_empty());
  assert!(parsed.errors.is_empty());
  assert!(lowered.errors.is_empty());
  assert!(cx.errors.is_empty());
  ret
}

#[test]
fn t() {
  let (cx, _) = get();
  assert!(cx.errors.is_empty());
}
