use std::borrow::Borrow;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
  pub fn new<S>(s: S) -> Self
  where
    S: Into<String>,
  {
    Self(s.into())
  }
}

impl Borrow<str> for Name {
  fn borrow(&self) -> &str {
    self.0.borrow()
  }
}

impl PartialEq<str> for Name {
  fn eq(&self, other: &str) -> bool {
    self.0 == other
  }
}

impl fmt::Display for Name {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
