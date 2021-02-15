pub(crate) const INDENT: &str = "  ";

#[derive(Default)]
pub(crate) struct Cx {
  buf: String,
}

impl Cx {
  pub(crate) fn push(&mut self, s: &str) {
    self.buf.push_str(s);
  }

  pub(crate) fn finish(self) -> String {
    self.buf
  }
}
