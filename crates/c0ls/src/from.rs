/// Crate-owned copy-paste of [`core::convert::From`].
///
/// The orphan rules prevent us from implementing `From<T>` for `U` when both
/// `T` and `U` are types both not from the current crate, since the `From`
/// trait itself is also not from the current crate.
///
/// But if a trait is owned by the current crate, we can implement it on
/// whatever we want, in the scope of that crate.
pub(crate) trait CrateFrom<T>: Sized {
  fn from(_: T) -> Self;
}

impl CrateFrom<lsp_types::Position> for analysis::Position {
  fn from(val: lsp_types::Position) -> Self {
    Self {
      line: val.line,
      character: val.character,
    }
  }
}

impl CrateFrom<analysis::Position> for lsp_types::Position {
  fn from(val: analysis::Position) -> Self {
    Self {
      line: val.line,
      character: val.character,
    }
  }
}

impl CrateFrom<analysis::Range> for lsp_types::Range {
  fn from(val: analysis::Range) -> Self {
    Self {
      start: CrateFrom::from(val.start),
      end: CrateFrom::from(val.end),
    }
  }
}

impl CrateFrom<lsp_types::Range> for analysis::Range {
  fn from(val: lsp_types::Range) -> Self {
    Self {
      start: CrateFrom::from(val.start),
      end: CrateFrom::from(val.end),
    }
  }
}

impl CrateFrom<analysis::Location> for lsp_types::Location {
  fn from(val: analysis::Location) -> Self {
    Self {
      uri: val.uri,
      range: CrateFrom::from(val.range),
    }
  }
}

impl CrateFrom<lsp_types::Location> for analysis::Location {
  fn from(val: lsp_types::Location) -> Self {
    Self {
      uri: val.uri,
      range: CrateFrom::from(val.range),
    }
  }
}

impl CrateFrom<analysis::Hover> for lsp_types::Hover {
  fn from(val: analysis::Hover) -> Self {
    Self {
      range: Some(CrateFrom::from(val.range)),
      contents: lsp_types::HoverContents::Markup(lsp_types::MarkupContent {
        kind: lsp_types::MarkupKind::Markdown,
        value: val.contents.to_string(),
      }),
    }
  }
}

impl CrateFrom<analysis::Diagnostic> for lsp_types::Diagnostic {
  fn from(val: analysis::Diagnostic) -> Self {
    Self {
      range: CrateFrom::from(val.range),
      severity: None,
      code: None,
      code_description: None,
      source: None,
      message: val.message,
      related_information: None,
      tags: None,
      data: None,
    }
  }
}
