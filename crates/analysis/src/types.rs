use std::fmt;
use text_pos::Range;
use uri_db::Uri;

#[derive(Debug)]
pub struct Location {
  pub uri: Uri,
  pub range: Range,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Diagnostic {
  pub range: Range,
  pub message: String,
}

impl fmt::Display for Diagnostic {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}: {}", self.range, self.message)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CodeBlock(String);

impl CodeBlock {
  pub fn new(s: String) -> Self {
    Self(s)
  }
}

impl fmt::Display for CodeBlock {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "```c0\n{}\n```", self.0)
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hover {
  pub contents: CodeBlock,
  pub range: Range,
}

#[derive(Debug)]
pub struct Edit {
  pub text: String,
  pub range: Option<Range>,
}

#[derive(Debug)]
pub enum Update {
  Create(Uri, String),
  Delete(Uri),
}
