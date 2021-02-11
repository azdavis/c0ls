use crate::uri::Uri;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
  /// zero-based
  pub line: u32,
  pub character: u32,
}

impl fmt::Display for Position {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}", self.line + 1, self.character + 1)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
  pub start: Position,
  pub end: Position,
}

impl fmt::Display for Range {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}-{}", self.start, self.end)
  }
}

#[derive(Debug)]
pub struct Location {
  pub uri: Uri,
  pub range: Range,
}

#[derive(Debug)]
pub struct Diagnostic {
  pub range: Range,
  pub message: String,
}

impl fmt::Display for Diagnostic {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}: {}", self.range, self.message)
  }
}

#[derive(Debug)]
pub struct Markdown(String);

impl Markdown {
  pub fn new(s: String) -> Self {
    Self(s)
  }

  pub fn into_string(self) -> String {
    self.0
  }
}

#[derive(Debug)]
pub struct Hover {
  pub contents: Markdown,
  pub range: Range,
}