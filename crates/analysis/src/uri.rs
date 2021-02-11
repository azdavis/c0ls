use statics::{FileId, FileKind};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Uri(PathBuf);

impl Uri {
  pub fn new(path: PathBuf) -> Self {
    Self(path)
  }

  pub fn as_path(&self) -> &Path {
    self.0.as_path()
  }
}

impl Borrow<Path> for Uri {
  fn borrow(&self) -> &Path {
    self.as_path()
  }
}

#[derive(Debug, Default)]
pub(crate) struct Map {
  id_to_uri: HashMap<FileId, Uri>,
  uri_to_id: HashMap<Uri, FileId>,
  next: u32,
}

impl Map {
  pub(crate) fn insert(&mut self, uri: Uri, kind: FileKind) -> FileId {
    if let Some(ret) = self.get_id(&uri) {
      return ret;
    }
    let ret = FileId::new(self.next, kind);
    self.next += 1;
    assert!(self.id_to_uri.insert(ret, uri.clone()).is_none());
    assert!(self.uri_to_id.insert(uri, ret).is_none());
    ret
  }

  pub(crate) fn get_id<Q>(&self, key: &Q) -> Option<FileId>
  where
    Uri: Borrow<Q>,
    Q: ?Sized + Hash + Eq,
  {
    self.uri_to_id.get(key).copied()
  }

  pub(crate) fn get(&self, file_id: FileId) -> &Uri {
    self.id_to_uri.get(&file_id).expect("no uri for file id")
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = FileId> + '_ {
    self.id_to_uri.keys().copied()
  }
}

impl Index<FileId> for Map {
  type Output = Uri;
  fn index(&self, index: FileId) -> &Self::Output {
    self.get(index)
  }
}
