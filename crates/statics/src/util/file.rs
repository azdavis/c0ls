#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileKind {
  Source,
  Header,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId {
  raw: u32,
}

const TOP: u32 = 1 << 31;

impl FileId {
  /// Panics if the MSB of `id` is 1.
  pub fn new(id: u32, kind: FileKind) -> Self {
    assert_eq!(TOP & id, 0);
    let raw = match kind {
      FileKind::Source => id | TOP,
      FileKind::Header => id,
    };
    Self { raw }
  }
}

impl FileId {
  pub(crate) fn kind(&self) -> FileKind {
    if (self.raw & TOP) == TOP {
      FileKind::Source
    } else {
      FileKind::Header
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InFile<T> {
  pub file: FileId,
  pub val: T,
}

impl<T> InFile<T> {
  pub fn new(file: FileId, val: T) -> Self {
    Self { file, val }
  }

  pub fn wrap<U>(self, val: U) -> InFile<U> {
    InFile::new(self.file, val)
  }
}
