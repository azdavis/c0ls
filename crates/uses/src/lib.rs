//! Resolves `#use` pragmas.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

use std::fmt;
use std::path::{Component, Path, PathBuf};
use std_lib::Lib;
use syntax::rowan::TextRange;
use uri_db::{UriDb, UriId};

/// The processed use pragmas.
#[derive(Debug, Default)]
pub struct Uses {
  /// The uses.
  pub uses: Vec<Use>,
  /// The errors encountered.
  pub errors: Vec<Error>,
}

/// Translates the `uses` for the file at the given `id` into fully resolved
/// uses.
pub fn get(uris: &UriDb, id: UriId, uses: Vec<syntax::Use<'_>>) -> Uses {
  let mut ret = Uses::default();
  for u in uses {
    let range = u.range;
    match get_one(uris, id, u.path, u.kind) {
      Ok(kind) => ret.uses.push(Use { range, kind }),
      Err(kind) => ret.errors.push(Error { range, kind }),
    }
  }
  ret
}

fn get_one(
  uris: &UriDb,
  id: UriId,
  path: &str,
  kind: syntax::UseKind,
) -> Result<UseKind, ErrorKind> {
  match kind {
    syntax::UseKind::Local => {
      let uri = uris.get(id);
      let mut buf = PathBuf::from(uri.path()).parent().unwrap().to_owned();
      for c in Path::new(path).components() {
        match c {
          Component::Prefix(_) | Component::RootDir => {
            return Err(ErrorKind::AbsolutePath)
          }
          Component::CurDir => {}
          Component::ParentDir => {
            if !buf.pop() {
              return Err(ErrorKind::NoSuchPath);
            }
          }
          Component::Normal(s) => buf.push(s),
        }
      }
      let mut new_uri = uri.clone();
      new_uri.set_path(buf.as_os_str().to_str().unwrap());
      match uris.get_id(&new_uri) {
        Some(x) => Ok(UseKind::File(x)),
        None => Err(ErrorKind::NoSuchPath),
      }
    }
    syntax::UseKind::Lib => match path.parse::<Lib>() {
      Ok(lib) => Ok(UseKind::Lib(lib)),
      Err(()) => Err(ErrorKind::NoSuchLib),
    },
  }
}

/// A use.
#[derive(Debug)]
pub struct Use {
  /// The kind of use.
  pub kind: UseKind,
  /// The text range of the original pragma.
  pub range: TextRange,
}

/// A kind of use.
#[derive(Debug, Clone, Copy)]
pub enum UseKind {
  /// A file use, like `use "foo.h0"`.
  File(UriId),
  /// A lib use, like `#use <conio>`.
  Lib(Lib),
}

/// An error when translating uses.
#[derive(Debug)]
pub struct Error {
  /// The kind of error.
  pub kind: ErrorKind,
  /// The range of the problematic pragma.
  pub range: TextRange,
}

/// A kind of error.
#[derive(Debug)]
pub enum ErrorKind {
  /// No such library, e.g. `#use <foo>`.
  NoSuchLib,
  /// No such path, e.g. `#use "nope.java"`.
  NoSuchPath,
  /// An absolute path, e.g. `#use "/tmp/foo.c0"`.
  AbsolutePath,
}

impl fmt::Display for ErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      ErrorKind::NoSuchLib => write!(f, "no such lib"),
      ErrorKind::NoSuchPath => write!(f, "no such path"),
      ErrorKind::AbsolutePath => write!(f, "cannot use absolute path"),
    }
  }
}
