//! Positions in text.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use std::fmt;
use text_size::{TextRange, TextSize};

#[derive(Debug)]
struct Line {
  end: TextSize,
  /// pairs of (where this char was in the line, the difference between the
  /// number of bytes needed to represent this char in utf8 and utf16)
  non_ascii: Vec<(TextSize, u32)>,
}

#[derive(Debug)]
pub struct PositionDb {
  lines: Vec<Line>,
}

impl PositionDb {
  pub fn new(s: &str) -> Self {
    let mut end = TextSize::from(0);
    let mut col = TextSize::from(0);
    let mut lines = Vec::new();
    let mut non_ascii = Vec::new();
    for c in s.chars() {
      if !c.is_ascii() {
        // it should never happen that for a given c, the len_utf16 for c is
        // greater than the len_utf8 for c.
        let diff = c.len_utf8() - c.len_utf16();
        non_ascii.push((col, diff as u32));
      }
      if c == '\n' {
        lines.push(Line { end, non_ascii });
        non_ascii = Vec::new();
        col = TextSize::from(0);
      }
      let ts = TextSize::of(c);
      end += ts;
      col += ts;
    }
    lines.push(Line { end, non_ascii });
    lines.shrink_to_fit();
    Self { lines }
  }

  pub fn position(&self, pos: TextSize) -> Position {
    let line = self.lines.iter().position(|line| pos <= line.end).unwrap();
    let pos = match line.checked_sub(1) {
      None => pos,
      Some(prev) => pos - self.start(prev),
    };
    let mut character = u32::from(pos);
    for &(idx, diff) in self.lines[line].non_ascii.iter() {
      if idx < pos {
        character -= diff;
      } else {
        break;
      }
    }
    Position {
      line: line as u32,
      character,
    }
  }

  pub fn text_size(&self, pos: Position) -> TextSize {
    let line = pos.line as usize;
    let start = line
      .checked_sub(1)
      .map_or(TextSize::from(0), |line| self.start(line));
    let mut col = pos.character;
    for &(idx, diff) in self.lines[line].non_ascii.iter() {
      if u32::from(idx) < col {
        col += diff;
      } else {
        break;
      }
    }
    start + TextSize::from(col)
  }

  pub fn range(&self, rng: TextRange) -> Range {
    Range {
      start: self.position(rng.start()),
      end: self.position(rng.end()),
    }
  }

  fn start(&self, line: usize) -> TextSize {
    // 1 for the newline
    self.lines[line].end + TextSize::from(1)
  }
}

#[cfg(test)]
mod tests {
  use super::{Position, PositionDb, TextSize};

  fn check(s: &str, tests: &[(u32, u32, u32)]) {
    let lines = PositionDb::new(s);
    for &(idx, line, character) in tests {
      let text_size = TextSize::from(idx);
      let pos = Position { line, character };
      assert_eq!(lines.position(text_size), pos);
      assert_eq!(lines.text_size(pos), text_size);
    }
  }

  #[test]
  fn simple() {
    check(
      "hello\nnew\nworld\n",
      &[
        (0, 0, 0),
        (1, 0, 1),
        (4, 0, 4),
        (5, 0, 5),
        (6, 1, 0),
        (9, 1, 3),
        (10, 2, 0),
        (11, 2, 1),
        (15, 2, 5),
        (16, 3, 0),
      ],
    );
  }

  #[test]
  fn leading_newline() {
    check(
      "\n\nhey\n\nthere",
      &[
        (0, 0, 0),
        (1, 1, 0),
        (2, 2, 0),
        (3, 2, 1),
        (5, 2, 3),
        (6, 3, 0),
        (7, 4, 0),
        (8, 4, 1),
        (12, 4, 5),
      ],
    );
  }

  #[test]
  fn lsp_spec_example() {
    check(
      "a𐐀b",
      &[
        (0, 0, 0),
        (1, 0, 1),
        // 2, 3, 4 impossible because 𐐀 is 4 bytes long in UTF-8
        (5, 0, 3),
        (6, 0, 4),
      ],
    );
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Range {
  pub start: Position,
  pub end: Position,
}

impl fmt::Display for Range {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}-{}", self.start, self.end)
  }
}
