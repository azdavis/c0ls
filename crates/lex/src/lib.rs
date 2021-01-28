//! Lexes a string into tokens.

#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use std::convert::TryInto;
use syntax::event_parse::Token;
use syntax::rowan::{TextRange, TextSize};
use syntax::SyntaxKind as SK;

#[derive(Debug)]
pub struct Lex<'input> {
  pub tokens: Vec<Token<'input, SK>>,
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
  EmptyHexLit,
  UnclosedStringLit,
  UnclosedCharLit,
  UnclosedLibLit,
  WrongLenCharLit(usize),
  InvalidCharEscape,
  InvalidSource,
}

pub fn get(s: &str) -> Lex<'_> {
  let bs = s.as_bytes();
  let mut tokens = Vec::new();
  let mut cx = Cx::default();
  while cx.i < bs.len() {
    let start = cx.i;
    let kind = go(&mut cx, bs);
    let text = std::str::from_utf8(&bs[start..cx.i]).unwrap();
    tokens.push(Token { kind, text });
  }
  Lex {
    tokens,
    errors: cx.errors,
  }
}

#[derive(Default)]
struct Cx {
  errors: Vec<LexError>,
  i: usize,
  saw_use: bool,
}

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
        while let Some(&b) = bs.get(cx.i) {
          cx.i += 1;
          if b == b'\n' {
            break;
          }
        }
        return SK::LineComment;
      }
      // not a comment
      _ => {}
    }
  }
  // whitespace
  if whitespace(b).is_some() {
    while let Some(w) = bs.get(cx.i).copied().and_then(whitespace) {
      cx.i += 1;
      if matches!(w, Whitespace::Newline) {
        cx.saw_use = false;
      }
    }
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
        }
        SK::HexLit
      } else {
        SK::DecLit
      }
    } else {
      // dec
      advance_while(cx, bs, u8::is_ascii_digit);
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
              err(cx, cx.i - 2, LexErrorKind::InvalidCharEscape);
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
              err(cx, cx.i - 2, LexErrorKind::InvalidCharEscape);
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
  // lib lit
  if b == b'<' && cx.saw_use {
    cx.i += 1;
    let closed = loop {
      let b = match bs.get(cx.i) {
        None => break false,
        Some(&b) => b,
      };
      cx.i += 1;
      if b == b'>' {
        break true;
      }
    };
    if !closed {
      err(cx, start, LexErrorKind::UnclosedLibLit);
    }
    return SK::LibLit;
  }
  // punctuation
  for &(sk_text, sk) in SK::PUNCTUATION.iter() {
    if bs.get(cx.i..cx.i + sk_text.len()) == Some(sk_text) {
      cx.i += sk_text.len();
      return sk;
    }
  }
  // #use
  if bs.get(cx.i..cx.i + USE.len()) == Some(USE) {
    cx.i += USE.len();
    cx.saw_use = true;
    return SK::UseKw;
  }
  // invalid char. go until we find a valid str. this should terminate before
  // cx.i goes past the end of bs because bs comes from a str.
  loop {
    match std::str::from_utf8(&bs[start..cx.i]) {
      Ok(_) => break,
      Err(_) => cx.i += 1,
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
    range: TextRange::new(text_size(start), text_size(cx.i)),
    kind,
  });
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

const USE: &[u8] = b"#use";
