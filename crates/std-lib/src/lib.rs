//! The C0 standard libraries.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

use statics::{Cx, EnvWithIds, Import};
use std::str::FromStr;

/// The standard libraries.
#[derive(Debug)]
pub struct StdLib {
  args: EnvWithIds,
  conio: EnvWithIds,
  file: EnvWithIds,
  img: EnvWithIds,
  parse: EnvWithIds,
  rand: EnvWithIds,
  string: EnvWithIds,
  util: EnvWithIds,
}

impl StdLib {
  /// Returns the environment of the given library.
  pub fn get(&self, lib: Lib) -> &EnvWithIds {
    match lib {
      Lib::Args => &self.args,
      Lib::Conio => &self.conio,
      Lib::File => &self.file,
      Lib::Img => &self.img,
      Lib::Parse => &self.parse,
      Lib::Rand => &self.rand,
      Lib::String => &self.string,
      Lib::Util => &self.util,
    }
  }
}

/// A standard library.
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub enum Lib {
  Args,
  Conio,
  File,
  Img,
  Parse,
  Rand,
  String,
  Util,
}

impl FromStr for Lib {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let ret = match s {
      "args" => Self::Args,
      "conio" => Self::Conio,
      "file" => Self::File,
      "img" => Self::Img,
      "parse" => Self::Parse,
      "rand" => Self::Rand,
      "string" => Self::String,
      "util" => Self::Util,
      _ => return Err(()),
    };
    Ok(ret)
  }
}

/// The [`Cx`] will always have no errors. The `TyDb` inside the `Cx` allows
/// getting information about the returned `Env`s inside the [`StdLib`].
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

fn get_one(cx: &mut Cx, contents: &str) -> EnvWithIds {
  let lexed = lex::get(contents);
  let parsed = parse::get(lexed.tokens);
  let lowered = lower::get(parsed.root);
  let ret = statics::get(cx, &Import::default(), false, &lowered.root);
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
