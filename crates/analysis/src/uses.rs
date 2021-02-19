use std::fmt;
use std::path::{Component, Path, PathBuf};
use std_lib::Lib;
use syntax::rowan::TextRange;
use uri_db::{UriDb, UriId};

#[derive(Debug, Default)]
pub(crate) struct Uses {
  pub(crate) uses: Vec<Use>,
  pub(crate) errors: Vec<Error>,
}

pub(crate) fn get(uris: &UriDb, id: UriId, uses: Vec<lex::Use<'_>>) -> Uses {
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
  kind: lex::UseKind,
) -> Result<UseKind, ErrorKind> {
  match kind {
    lex::UseKind::Local => {
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
    lex::UseKind::Lib => match path.parse::<Lib>() {
      Ok(lib) => Ok(UseKind::Lib(lib)),
      Err(()) => Err(ErrorKind::NoSuchLib),
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

#[derive(Debug)]
pub(crate) struct Error {
  pub kind: ErrorKind,
  pub range: TextRange,
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
