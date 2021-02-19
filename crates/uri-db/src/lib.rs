//! A database of URIs of C0 source files.
//!
//! This depends on the `url` crate, but we call them "URIs". Basically, we're
//! just following what `lsp-types` calls them.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

pub use url::Url as Uri;

use rustc_hash::FxHashMap;
use std::borrow::Borrow;
use std::hash::Hash;
use std::ops::Index;
use std::path::Path;

/// A URI database.
#[derive(Debug, Default)]
pub struct UriDb {
  id_to_uri: FxHashMap<UriId, Uri>,
  uri_to_id: FxHashMap<Uri, UriId>,
  next: u32,
}

impl UriDb {
  /// Inserts a URI into the database.
  ///
  /// Panics if the URI does not have the extension `c0` or `h0`, or if there
  /// are way too many URIs in the database (like, nearing `u32::MAX` many).
  pub fn insert(&mut self, uri: Uri) -> UriId {
    if let Some(ret) = self.get_id(&uri) {
      return ret;
    }
    let ext = Path::new(uri.path())
      .extension()
      .expect("no extension")
      .to_str()
      .expect("extension is not UTF-8");
    let kind = match ext {
      "h0" => UriKind::Header,
      "c0" => UriKind::Source,
      _ => panic!("bad extension: {}", ext),
    };
    let ret = UriId::new(self.next, kind);
    self.next += 1;
    assert!(self.id_to_uri.insert(ret, uri.clone()).is_none());
    assert!(self.uri_to_id.insert(uri, ret).is_none());
    ret
  }

  /// Returns the ID associated with this URI.
  pub fn get_id<Q>(&self, key: &Q) -> Option<UriId>
  where
    Uri: Borrow<Q>,
    Q: ?Sized + Hash + Eq,
  {
    self.uri_to_id.get(key).copied()
  }

  /// Returns the URI associated with this ID.
  pub fn get(&self, file_id: UriId) -> &Uri {
    self.id_to_uri.get(&file_id).expect("no uri for id")
  }

  /// Returns an iterator over the IDs.
  pub fn iter(&self) -> impl Iterator<Item = UriId> + '_ {
    self.id_to_uri.keys().copied()
  }
}

impl Index<UriId> for UriDb {
  type Output = Uri;
  fn index(&self, index: UriId) -> &Self::Output {
    self.get(index)
  }
}

/// A URI identifier.
///
/// Yes, this is a "uniform resource identifier identifier". We only use this to
/// avoid cloning URIs all over the place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UriId {
  raw: u32,
}

const TOP: u32 = 1 << 31;

impl UriId {
  /// Panics if the MSB of `id` is 1.
  fn new(id: u32, kind: UriKind) -> Self {
    assert_eq!(TOP & id, 0);
    let raw = match kind {
      UriKind::Source => id | TOP,
      UriKind::Header => id,
    };
    Self { raw }
  }
}

impl UriId {
  /// Returns the kind of the URI that this is an ID for.
  pub fn kind(&self) -> UriKind {
    if (self.raw & TOP) == TOP {
      UriKind::Source
    } else {
      UriKind::Header
    }
  }
}

/// A kind of C0 file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UriKind {
  /// A source file, with extension `c0`.
  Source,
  /// A header file, with extension `h0`.
  Header,
}
