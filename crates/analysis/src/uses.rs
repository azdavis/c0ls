//! TODO make this its own crate?

use crate::uri::Map;
use statics_neue::FileId;
use std::fmt;
use std::path::{Component, PathBuf};

pub(crate) fn get(map: &Map, id: FileId, u: syntax::Use) -> Result<Use, Error> {
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
  map: &Map,
  id: FileId,
  path: String,
  kind: syntax::UseKind,
) -> Result<UseKind, ErrorKind> {
  match kind {
    syntax::UseKind::Local => {
      let mut buf = map.get(id).as_path().parent().unwrap().to_owned();
      let fella = PathBuf::from(path);
      for c in fella.components() {
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
      match map.get_id(buf.as_path()) {
        Some(x) => Ok(UseKind::File(x)),
        None => Err(ErrorKind::NoSuchPath),
      }
    }
    syntax::UseKind::Lib => match path.as_str() {
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

#[derive(Debug)]
pub(crate) enum UseKind {
  File(FileId),
  Lib(Lib),
}

#[derive(Debug)]
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
