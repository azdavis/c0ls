#![deny(missing_debug_implementations)]
#![deny(rust_2018_idioms)]

use std::ops::Range;
use syntax::event_parse::Token;
use syntax::SyntaxKind as SK;

#[derive(Debug)]
pub struct Lex<'input> {
  pub tokens: Vec<Token<'input, SK>>,
  pub errors: Vec<LexError>,
}

#[derive(Debug)]
pub struct LexError {
  pub range: Range<usize>,
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

pub fn lex(s: &str) -> Lex<'_> {
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
    let text = &bs[start..cx.i];
    let kind = KEYWORDS
      .iter()
      .find_map(|&(sk_text, sk)| if sk_text == text { Some(sk) } else { None })
      .unwrap_or(SK::Ident);
    return kind;
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
  for &(sk_text, sk) in PUNCTUATION.iter() {
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
    range: start..cx.i,
    kind,
  });
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

// sorted in length-lex order (i think that's what it's called?)

const KEYWORDS: [(&[u8], SK); 21] = [
  // 11
  (b"alloc_array", SK::AllocArrayKw),
  // 8
  (b"continue", SK::ContinueKw),
  // 7
  (b"typedef", SK::TypedefKw),
  // 6
  (b"assert", SK::AssertKw),
  (b"return", SK::ReturnKw),
  (b"string", SK::StringKw),
  (b"struct", SK::StructKw),
  // 5
  (b"alloc", SK::AllocKw),
  (b"break", SK::BreakKw),
  (b"error", SK::ErrorKw),
  (b"false", SK::FalseKw),
  (b"while", SK::WhileKw),
  // 4
  (b"bool", SK::BoolKw),
  (b"char", SK::CharKw),
  (b"else", SK::ElseKw),
  (b"NULL", SK::NullKw),
  (b"true", SK::TrueKw),
  (b"void", SK::VoidKw),
  // 3
  (b"for", SK::ForKw),
  (b"int", SK::IntKw),
  // 2
  (b"if", SK::IfKw),
];

const PUNCTUATION: [(&[u8], SK); 45] = [
  // 3
  (b"<<=", SK::LtLtEq),
  (b">>=", SK::GtGtEq),
  // 2
  (b"--", SK::MinusMinus),
  (b"-=", SK::MinusEq),
  (b"->", SK::Arrow),
  (b"!=", SK::BangEq),
  (b"*=", SK::StarEq),
  (b"/=", SK::SlashEq),
  (b"&&", SK::AndAnd),
  (b"&=", SK::AndEq),
  (b"%=", SK::PercentEq),
  (b"^=", SK::CaratEq),
  (b"++", SK::PlusPlus),
  (b"+=", SK::PlusEq),
  (b"<<", SK::LtLt),
  (b"<=", SK::LtEq),
  (b"==", SK::EqEq),
  (b">=", SK::GtEq),
  (b">>", SK::GtGt),
  (b"|=", SK::BarEq),
  (b"||", SK::BarBar),
  // 1
  (b"-", SK::Minus),
  (b",", SK::Comma),
  (b";", SK::Semicolon),
  (b":", SK::Colon),
  (b"!", SK::Bang),
  (b"?", SK::Question),
  (b".", SK::Dot),
  (b"(", SK::LRound),
  (b")", SK::RRound),
  (b"[", SK::LSquare),
  (b"]", SK::RSquare),
  (b"{", SK::LCurly),
  (b"}", SK::RCurly),
  (b"*", SK::Star),
  (b"/", SK::Slash),
  (b"&", SK::And),
  (b"%", SK::Percent),
  (b"^", SK::Carat),
  (b"+", SK::Plus),
  (b"<", SK::Lt),
  (b"=", SK::Eq),
  (b">", SK::Gt),
  (b"|", SK::Bar),
  (b"~", SK::Tilde),
];

const USE: &[u8] = b"#use";
