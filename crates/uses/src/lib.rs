//! Resolves `#use` pragmas.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use std::fmt;
use std::path::{Component, Path, PathBuf};
use std_lib::Lib;
use syntax::rowan::TextRange;
use uri_db::{UriDb, UriId};

#[derive(Debug, Default)]
pub struct Uses {
  pub uses: Vec<Use>,
  pub errors: Vec<Error>,
}

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

#[derive(Debug)]
pub struct Use {
  pub kind: UseKind,
  pub range: TextRange,
}

#[derive(Debug, Clone, Copy)]
pub enum UseKind {
  File(UriId),
  Lib(Lib),
}

#[derive(Debug)]
pub struct Error {
  pub kind: ErrorKind,
  pub range: TextRange,
}

#[derive(Debug)]
pub enum ErrorKind {
  NoSuchLib,
  NoSuchPath,
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
