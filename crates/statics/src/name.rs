use std::borrow::Borrow;

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
