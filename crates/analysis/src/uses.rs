use crate::uri::{UriDb, UriId};
use std::fmt;
use std::path::{Component, Path, PathBuf};

pub(crate) fn get(
  map: &UriDb,
  id: UriId,
  u: lex::Use<'_>,
) -> Result<Use, Error> {
  match get_impl(map, id, u.path, u.kind) {
    Ok(kind) => Ok(Use {
      range: u.range,
      kind,
    }),
    Err(kind) => Err(Error {
      range: u.range,
      kind,
    }),
  }
}

fn get_impl(
  map: &UriDb,
  id: UriId,
  path: &str,
  kind: lex::UseKind,
) -> Result<UseKind, ErrorKind> {
  match kind {
    lex::UseKind::Local => {
      let uri = map.get(id);
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
      match map.get_id(&new_uri) {
        Some(x) => Ok(UseKind::File(x)),
        None => Err(ErrorKind::NoSuchPath),
      }
    }
    lex::UseKind::Lib => match path {
      "args" => Ok(UseKind::Lib(Lib::Args)),
      "conio" => Ok(UseKind::Lib(Lib::Conio)),
      "file" => Ok(UseKind::Lib(Lib::File)),
      "img" => Ok(UseKind::Lib(Lib::Img)),
      "parse" => Ok(UseKind::Lib(Lib::Parse)),
      "rand" => Ok(UseKind::Lib(Lib::Rand)),
      "string" => Ok(UseKind::Lib(Lib::String)),
      "util" => Ok(UseKind::Lib(Lib::Util)),
      _ => Err(ErrorKind::NoSuchLib),
    },
  }
}

#[derive(Debug)]
pub(crate) struct Use {
  pub kind: UseKind,
  pub range: syntax::rowan::TextRange,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum UseKind {
  File(UriId),
  Lib(Lib),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Lib {
  Args,
  Conio,
  File,
  Img,
  Parse,
  Rand,
  String,
  Util,
}

#[derive(Debug)]
pub(crate) struct Error {
  pub kind: ErrorKind,
  pub range: syntax::rowan::TextRange,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
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
