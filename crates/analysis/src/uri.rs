use rustc_hash::FxHashMap;
use statics::FileKind;
use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Index;
use std::path::Path;
use url::Url;

#[derive(Debug, Default)]
pub(crate) struct UriDb {
  id_to_uri: FxHashMap<UriId, Url>,
  uri_to_id: FxHashMap<Url, UriId>,
  next: u32,
}

impl UriDb {
  pub(crate) fn insert(&mut self, uri: Url) -> UriId {
    if let Some(ret) = self.get_id(&uri) {
      return ret;
    }
    let ext = Path::new(uri.path())
      .extension()
      .expect("no extension")
      .to_str()
      .expect("extension is not UTF-8");
    let kind = match ext {
      "h0" => FileKind::Header,
      _ => FileKind::Source,
    };
    let ret = UriId::new(self.next, kind);
    self.next += 1;
    assert!(self.id_to_uri.insert(ret, uri.clone()).is_none());
    assert!(self.uri_to_id.insert(uri, ret).is_none());
    ret
  }

  pub(crate) fn get_id<Q>(&self, key: &Q) -> Option<UriId>
  where
    Url: Borrow<Q>,
    Q: ?Sized + Hash + Eq,
  {
    self.uri_to_id.get(key).copied()
  }

  pub(crate) fn get(&self, file_id: UriId) -> &Url {
    self.id_to_uri.get(&file_id).expect("no uri for file id")
  }

  pub(crate) fn iter(&self) -> impl Iterator<Item = UriId> + '_ {
    self.id_to_uri.keys().copied()
  }
}

impl Index<UriId> for UriDb {
  type Output = Url;
  fn index(&self, index: UriId) -> &Self::Output {
    self.get(index)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UriId {
  raw: u32,
}

const TOP: u32 = 1 << 31;

impl UriId {
  /// Panics if the MSB of `id` is 1.
  fn new(id: u32, kind: FileKind) -> Self {
    assert_eq!(TOP & id, 0);
    let raw = match kind {
      FileKind::Source => id | TOP,
      FileKind::Header => id,
    };
    Self { raw }
  }
}

impl UriId {
  pub(crate) fn kind(&self) -> FileKind {
    if (self.raw & TOP) == TOP {
      FileKind::Source
    } else {
      FileKind::Header
    }
  }
}
