use rustc_hash::FxHashMap;
use statics::{FileId, FileKind};
use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Index;
use url::Url;

#[derive(Debug, Default)]
pub(crate) struct UriDb {
  id_to_uri: FxHashMap<FileId, Url>,
  uri_to_id: FxHashMap<Url, FileId>,
  next: u32,
}

impl UriDb {
  pub(crate) fn insert(&mut self, uri: Url, kind: FileKind) -> FileId {
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
    Url: Borrow<Q>,
    Q: ?Sized + Hash + Eq,
  {
    self.uri_to_id.get(key).copied()
  }

  pub(crate) fn get(&self, file_id: FileId) -> &Url {
    self.id_to_uri.get(&file_id).expect("no uri for file id")
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = FileId> + '_ {
    self.id_to_uri.keys().copied()
  }
}

impl Index<FileId> for UriDb {
  type Output = Url;
  fn index(&self, index: FileId) -> &Self::Output {
    self.get(index)
  }
}
