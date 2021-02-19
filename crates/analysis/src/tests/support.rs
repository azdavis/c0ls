use crate::{CodeBlock, Db, Diagnostic, Hover, Position, Range};
use rustc_hash::FxHashMap;
use uri_db::Uri;

pub(crate) fn check(s: &str) {
  let mut files = FxHashMap::default();
  let uri = Uri::from_file_path("/tmp/main.c0").unwrap();
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
  assert_eq!(
    want.diagnostics.len(),
    got.len(),
    "mismatched number of diagnostics"
  );
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
  for struct_use in want.struct_uses.iter() {
    let want = *want.struct_defs.get(&struct_use.0).unwrap();
    check_def(&db, &uri, want, struct_use.1.start);
  }
  for fn_use in want.fn_uses.iter() {
    let want = *want.fn_defs.get(&fn_use.0).unwrap();
    check_def(&db, &uri, want, fn_use.1.start);
  }
  for type_def_use in want.type_def_uses.iter() {
    let want = *want.type_def_defs.get(&type_def_use.0).unwrap();
    check_def(&db, &uri, want, type_def_use.1.start);
  }
}

fn check_def(db: &Db, uri: &Uri, want: Range, pos: Position) {
  let got_def = match db.go_to_def(uri, pos) {
    None => panic!("no def info at {}", pos),
    Some(x) => x,
  };
  assert_eq!(*uri, got_def.uri);
  assert_eq!(want, got_def.range);
}

#[derive(Debug, Default)]
struct Expectations {
  diagnostics: Vec<Diagnostic>,
  hovers: Vec<Hover>,
  no_hovers: Vec<Range>,
  struct_defs: FxHashMap<String, Range>,
  struct_uses: Vec<(String, Range)>,
  fn_defs: FxHashMap<String, Range>,
  fn_uses: Vec<(String, Range)>,
  type_def_defs: FxHashMap<String, Range>,
  type_def_uses: Vec<(String, Range)>,
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
  let mut ret = Expectations::default();
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
      "error" => ret.diagnostics.push(Diagnostic {
        range,
        message: content,
      }),
      "hover" => {
        if content == "<none>" {
          ret.no_hovers.push(range)
        } else {
          ret.hovers.push(Hover {
            range,
            contents: CodeBlock::new(content),
          })
        }
      }
      "struct-def" => assert!(ret.struct_defs.insert(content, range).is_none()),
      "struct-use" => ret.struct_uses.push((content, range)),
      "fn-def" => assert!(ret.fn_defs.insert(content, range).is_none()),
      "fn-use" => ret.fn_uses.push((content, range)),
      "type-def-def" => {
        assert!(ret.type_def_defs.insert(content, range).is_none())
      }
      "type-def-use" => ret.type_def_uses.push((content, range)),
      bad => panic!("unknown expectation kind: {}", bad),
    }
  }
  ret.diagnostics.sort_unstable();
  ret.hovers.sort_unstable();
  ret.no_hovers.sort_unstable();
  ret.struct_uses.sort_unstable();
  ret.fn_uses.sort_unstable();
  ret.type_def_uses.sort_unstable();
  ret
}
