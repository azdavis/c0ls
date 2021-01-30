//! Lexes a string into tokens, and parses pragmas.
//!
//! We parse pragmas here because we need to know a file's dependencies before
//! we can parse it. This is because in order to parse a file correctly, we need
//! to know which typedefs are in scope, because of the typedef-name: identifier
//! problem.
//!
//! Pragmas are also problematic (and thus handled here) because of library
//! literals. Namely, if we see `<`, some characters, and `>`, this might be a
//! library literal, but only if we just saw `#use` and didn't yet see a
//! newline.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use std::convert::TryInto;
use std::fmt;
use syntax::event_parse::Token;
use syntax::rowan::{TextRange, TextSize};
use syntax::{SyntaxKind as SK, Use, UseKind};

#[derive(Debug)]
pub struct Lex<'input> {
  pub tokens: Vec<Token<'input, SK>>,
  /// although the tokens returned borrow the input string, for the uses, we
  /// take ownership. this is because the tokens are going right to the parser
  /// (which end the borrow on the input string) but the uses are sticking
  /// around longer, and we don't want to have to hold on to the input string
  /// for that long.
  pub uses: Vec<Use>,
  pub errors: Vec<LexError>,
}

#[derive(Debug)]
pub struct LexError {
  pub range: TextRange,
  pub kind: LexErrorKind,
}

#[derive(Debug)]
pub enum LexErrorKind {
  UnclosedBlockComment,
  InvalidPragma,
  UnclosedPragmaLit,
  EmptyHexLit,
  UnclosedStringLit,
  UnclosedCharLit,
  WrongLenCharLit(usize),
  InvalidEscape,
  IntLitTooLarge,
  InvalidSource,
}

impl fmt::Display for LexErrorKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      LexErrorKind::UnclosedBlockComment => write!(f, "unclosed block comment"),
      LexErrorKind::InvalidPragma => write!(f, "invalid pragma"),
      LexErrorKind::UnclosedPragmaLit => write!(f, "unclosed pragma literal"),
      LexErrorKind::EmptyHexLit => write!(f, "empty hex literal"),
      LexErrorKind::UnclosedStringLit => write!(f, "unclosed string literal"),
      LexErrorKind::UnclosedCharLit => write!(f, "unclosed char literal"),
      LexErrorKind::WrongLenCharLit(n) => match n {
        0 => write!(f, "empty char literal"),
        _ => write!(f, "char literal too long"),
      },
      LexErrorKind::InvalidEscape => write!(f, "invalid escape"),
      LexErrorKind::IntLitTooLarge => write!(f, "integer literal too large"),
      LexErrorKind::InvalidSource => write!(f, "invalid source character"),
    }
  }
}

pub fn get(s: &str) -> Lex<'_> {
  let bs = s.as_bytes();
  let mut tokens = Vec::new();
  let mut cx = Cx::default();
  while cx.i < bs.len() {
    let start = cx.i;
    let kind = go(&mut cx, bs);
    // must always advance
    assert!(start < cx.i);
    let text = std::str::from_utf8(&bs[start..cx.i]).unwrap();
    tokens.push(Token { kind, text });
  }
  Lex {
    tokens,
    errors: cx.errors,
    uses: cx.uses,
  }
}

#[derive(Default)]
struct Cx {
  errors: Vec<LexError>,
  i: usize,
  uses: Vec<Use>,
}

const MAX: u32 = 1 << 31;

/// requires bs is a valid &str. returns sk and updates cx.i from start to end
/// such that bs[start..end] is a str and sk is the kind for that str.
#[inline]
fn go(cx: &mut Cx, bs: &[u8]) -> SK {
  let b = bs[cx.i];
  let start = cx.i;
  // comments
  if b == b'/' {
    match bs.get(cx.i + 1) {
      // block comment
      Some(&b'*') => {
        cx.i += 2;
        let mut level = 1_usize;
        loop {
          match (bs.get(cx.i), bs.get(cx.i + 1)) {
            (Some(&b'/'), Some(&b'*')) => {
              cx.i += 2;
              level += 1;
            }
            (Some(&b'*'), Some(&b'/')) => {
              cx.i += 2;
              level -= 1;
              if level == 0 {
                break;
              }
            }
            (None, None) => {
              err(cx, start, LexErrorKind::UnclosedBlockComment);
              break;
            }
            _ => cx.i += 1,
          }
        }
        return SK::BlockComment;
      }
      // line comment
      Some(&b'/') => {
        cx.i += 2;
        advance_while(cx, bs, |&b| b != b'\n');
        return SK::LineComment;
      }
      // not a comment
      _ => {}
    }
  }
  // pragmas. kind of gross: we end up doing a bit of parsing in the lexer.
  if b == b'#' {
    cx.i += 1;
    let old_i = cx.i;
    advance_while(cx, bs, u8::is_ascii_alphabetic);
    if !matches!(&bs[old_i..cx.i], b"use" | b"ref") {
      advance_while(cx, bs, |&b| b != b'\n');
      return SK::Pragma;
    }
    // #use (and #ref) pragma. first eat the non-newline whitespace after the
    // pragma starter.
    advance_while(cx, bs, |&b| {
      matches!(whitespace(b), Some(Whitespace::Other))
    });
    // should have either a double quote or < to start the literal.
    let (end, kind) = match bs.get(cx.i) {
      Some(&b'"') => (b'"', UseKind::Local),
      Some(&b'<') => (b'>', UseKind::Lib),
      _ => {
        err(cx, start, LexErrorKind::InvalidPragma);
        // give up
        return SK::Pragma;
      }
    };
    cx.i += 1;
    let start_lit = cx.i;
    // i think we're supposed to consider escapes here, but this doesn't. but
    // honestly, what absolute madman using C0 is going around putting escapable
    // characters in their filenames?
    let end_lit = loop {
      let b = match bs.get(cx.i) {
        None => break None,
        Some(&b) => b,
      };
      cx.i += 1;
      if b == end {
        break Some(cx.i - 1);
      }
      if b == b'\n' {
        break None;
      }
    };
    match end_lit {
      Some(end_lit) => {
        let use_str = std::str::from_utf8(&bs[start_lit..end_lit]).unwrap();
        cx.uses.push(Use {
          kind,
          range: range(start, cx.i),
          path: use_str.to_owned(),
        });
      }
      None => err(cx, start, LexErrorKind::UnclosedPragmaLit),
    }
    // eat the rest of the non-newline whitespace
    advance_while(cx, bs, |&b| {
      matches!(whitespace(b), Some(Whitespace::Other))
    });
    return SK::Pragma;
  }
  // whitespace
  if whitespace(b).is_some() {
    advance_while(cx, bs, |&b| whitespace(b).is_some());
    return SK::Whitespace;
  }
  // identifiers and keywords
  if b.is_ascii_alphabetic() || b == b'_' {
    cx.i += 1;
    advance_while(cx, bs, |&b| b.is_ascii_alphanumeric() || b == b'_');
    return SK::keyword(&bs[start..cx.i]).unwrap_or(SK::Ident);
  }
  // num lit (dec/hex)
  if b.is_ascii_digit() {
    cx.i += 1;
    return if b == b'0' {
      if matches!(bs.get(cx.i), Some(&b'x') | Some(&b'X')) {
        cx.i += 1;
        let old_i = cx.i;
        advance_while(cx, bs, u8::is_ascii_hexdigit);
        if old_i == cx.i {
          err(cx, start, LexErrorKind::EmptyHexLit);
        } else {
          let digits = std::str::from_utf8(&bs[old_i..cx.i]).unwrap();
          // this is different from dec lit, not sure why.
          if u32::from_str_radix(digits, 16).is_err() {
            err(cx, start, LexErrorKind::IntLitTooLarge);
          }
        }
        SK::HexLit
      } else {
        SK::DecLit
      }
    } else {
      advance_while(cx, bs, u8::is_ascii_digit);
      let digits = std::str::from_utf8(&bs[start..cx.i]).unwrap();
      let too_large = match u32::from_str_radix(digits, 10) {
        Ok(n) => n > MAX,
        Err(_) => true,
      };
      if too_large {
        err(cx, start, LexErrorKind::IntLitTooLarge);
      }
      SK::DecLit
    };
  }
  // string lit
  if b == b'"' {
    cx.i += 1;
    let closed = loop {
      let b = match bs.get(cx.i) {
        None => break false,
        Some(b) => b,
      };
      cx.i += 1;
      match b {
        b'"' => break true,
        b'\\' => match bs.get(cx.i) {
          None => break false,
          Some(&b) => {
            cx.i += 1;
            if !is_esc(b) {
              err(cx, cx.i - 2, LexErrorKind::InvalidEscape);
            }
          }
        },
        _ => {}
      }
    };
    if !closed {
      err(cx, start, LexErrorKind::UnclosedStringLit);
    }
    return SK::StringLit;
  }
  // char lit
  if b == b'\'' {
    cx.i += 1;
    let mut len = 0_usize;
    let closed = loop {
      let b = match bs.get(cx.i) {
        None => break false,
        Some(b) => b,
      };
      cx.i += 1;
      match b {
        b'\'' => break true,
        b'\\' => match bs.get(cx.i) {
          None => break false,
          Some(&b) => {
            cx.i += 1;
            if !is_esc(b) && b != b'0' {
              err(cx, cx.i - 2, LexErrorKind::InvalidEscape);
            }
          }
        },
        _ => {}
      }
      len += 1;
    };
    if !closed {
      err(cx, start, LexErrorKind::UnclosedCharLit);
    } else if len != 1 {
      err(cx, start, LexErrorKind::WrongLenCharLit(len));
    }
    return SK::CharLit;
  }
  // punctuation
  for &(sk_text, sk) in SK::PUNCTUATION.iter() {
    if bs.get(cx.i..cx.i + sk_text.len()) == Some(sk_text) {
      cx.i += sk_text.len();
      return sk;
    }
  }
  // invalid char. go until we find a valid str. this should terminate before
  // cx.i goes past the end of bs because bs comes from a str.
  loop {
    cx.i += 1;
    if std::str::from_utf8(&bs[start..cx.i]).is_ok() {
      break;
    }
  }
  err(cx, start, LexErrorKind::InvalidSource);
  SK::Invalid
}

fn advance_while(cx: &mut Cx, bs: &[u8], p: fn(&u8) -> bool) {
  while let Some(b) = bs.get(cx.i) {
    if p(b) {
      cx.i += 1;
    } else {
      break;
    }
  }
}

fn err(cx: &mut Cx, start: usize, kind: LexErrorKind) {
  cx.errors.push(LexError {
    range: range(start, cx.i),
    kind,
  });
}

fn range(start: usize, end: usize) -> TextRange {
  TextRange::new(text_size(start), text_size(end))
}

fn text_size(n: usize) -> TextSize {
  n.try_into().unwrap()
}

enum Whitespace {
  Newline,
  Other,
}

fn whitespace(b: u8) -> Option<Whitespace> {
  match b {
    b'\n' => Some(Whitespace::Newline),
    b'\r' | b' ' | b'\t' | 0xb | 0xc => Some(Whitespace::Other),
    _ => None,
  }
}

#[inline]
fn is_esc(b: u8) -> bool {
  matches!(
    b,
    b'n' | b't' | b'v' | b'b' | b'r' | b'f' | b'a' | b'\\' | b'\'' | b'"'
  )
}
