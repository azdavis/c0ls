use analysis::{url::Url, CodeBlock, Db, Diagnostic, Hover, Position, Range};
use rustc_hash::FxHashMap;

pub(crate) fn check(s: &str) {
  let mut files = FxHashMap::default();
  let uri = Url::from_file_path("/tmp/main.c0").unwrap();
  files.insert(uri.clone(), s.to_owned());
  let db = Db::new(files);
  let want = parse_expected(s);
  let mut got_all = db.all_diagnostics();
  let (got_uri, mut got) = got_all.pop().unwrap();
  assert!(got_all.is_empty());
  assert_eq!(uri, got_uri);
  got.sort_unstable();
  for (want, got) in want.diagnostics.iter().zip(got.iter()) {
    assert!(
      want.range == got.range,
      "mismatched ranges: want {}, got {} with message: '{}'",
      want.range,
      got.range,
      got.message,
    );
    assert!(
      got.message.starts_with(&want.message),
      "{}: '{}' does not start with '{}'",
      want.range,
      got.message,
      want.message
    );
  }
  assert_eq!(want.diagnostics.len(), got.len());
  for hover in want.hovers.iter() {
    let got_hover = match db.hover(&uri, hover.range.start) {
      None => panic!("no hover at {}", hover.range.start),
      Some(x) => x,
    };
    assert_eq!(*hover, got_hover);
  }
  for no_hover in want.no_hovers.iter() {
    assert!(db.hover(&uri, no_hover.start).is_none());
    assert!(db.hover(&uri, no_hover.end).is_none());
  }
}

struct Expectations {
  diagnostics: Vec<Diagnostic>,
  hovers: Vec<Hover>,
  no_hovers: Vec<Range>,
}

/// only supports ascii files, and treats all line comments as expectations.
/// thus an expectation is the following in sequence:
///
/// - the line comment start sigil, `//`
/// - zero or more spaces
/// - one or more `^`
/// - one space
/// - one or more non-`:` characters (the kind)
/// - one `:`
/// - one space
/// - one or more non-newline characters (the content)
/// - newline
///
/// the `^` determine the range of the expectation.
///
/// since the `^` are pointing to the previous line, an expectation comment
/// cannot be on the first line of the file.
#[allow(clippy::while_let_on_iterator)]
fn parse_expected(s: &str) -> Expectations {
  let mut cs = s.chars().peekable();
  let mut line: u32 = 0;
  let mut col: u32 = 0;
  let mut diagnostics = Vec::new();
  let mut hovers = Vec::new();
  let mut no_hovers = Vec::new();
  while let Some(c) = cs.next() {
    if c == '\n' {
      line += 1;
      col = 0;
      continue;
    }
    if c != '/' || cs.peek() != Some(&'/') {
      col += 1;
      continue;
    }
    cs.next();
    col += 1;
    while let Some(&c) = cs.peek() {
      if c == ' ' {
        cs.next();
        col += 1;
      } else {
        break;
      }
    }
    col += 1;
    let start = col;
    while let Some(&c) = cs.peek() {
      if c == '^' {
        cs.next();
        col += 1;
      } else {
        break;
      }
    }
    let range = Range {
      start: Position {
        line: line - 1,
        character: start,
      },
      end: Position {
        line: line - 1,
        character: col,
      },
    };
    assert_eq!(cs.next().unwrap(), ' ');
    col += 1;
    let mut kind = String::new();
    while let Some(c) = cs.next() {
      col += 1;
      if c == ':' {
        break;
      } else {
        kind.push(c);
      }
    }
    assert_eq!(cs.next().unwrap(), ' ');
    col += 1;
    let mut content = String::new();
    while let Some(&c) = cs.peek() {
      if c == '\n' {
        break;
      } else {
        content.push(c);
        cs.next();
        col += 1;
      }
    }
    match kind.as_str() {
      "error" => diagnostics.push(Diagnostic {
        range,
        message: content,
      }),
      "hover" => {
        if content == "<none>" {
          no_hovers.push(range)
        } else {
          hovers.push(Hover {
            range,
            contents: CodeBlock::new(content),
          })
        }
      }
      bad => panic!("unknown expectation kind: {}", bad),
    }
  }
  diagnostics.sort_unstable();
  hovers.sort_unstable();
  Expectations {
    diagnostics,
    hovers,
    no_hovers,
  }
}
