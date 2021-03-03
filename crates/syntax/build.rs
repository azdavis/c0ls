use identifier_case::snake_to_pascal;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::process::{Command, Stdio};
use syntax_gen::{gen, TokenKind};

const CONTENT: [(&str, &str); 6] = [
  ("Ident", "an identifier"),
  ("DecLit", "an integer literal"),
  ("HexLit", "a hexadecimal integer literal"),
  ("StringLit", "a string literal"),
  ("CharLit", "a char literal"),
  ("Pragma", "a pragma"),
];

fn get_kind(name: &str) -> (TokenKind, String) {
  if let Some(desc) = CONTENT
    .iter()
    .find_map(|&(n, desc)| (n == name).then(|| desc))
  {
    (TokenKind::Special(desc), name.to_owned())
  } else if name == "->" {
    (TokenKind::Punctuation, "Arrow".to_owned())
  } else if name.chars().any(|c| c.is_ascii_alphabetic()) {
    let mut ins = snake_to_pascal(name);
    ins.push_str("Kw");
    (TokenKind::Keyword, ins)
  } else {
    let mut ins = String::new();
    for c in name.chars() {
      let s = match char_name::get(c) {
        Some(x) => x,
        None => panic!("don't know the name for {}", c),
      };
      ins.push_str(s);
    }
    (TokenKind::Punctuation, ins)
  }
}

fn write_rust_file(name: &str, contents: &str) -> std::io::Result<()> {
  let mut proc = Command::new("rustfmt")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;
  proc.stdin.take().unwrap().write_all(contents.as_bytes())?;
  assert!(proc.wait()?.success());
  let mut stdout = proc.stdout.take().unwrap();
  let mut out_file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(name)?;
  std::io::copy(&mut stdout, &mut out_file)?;
  Ok(())
}

fn main() {
  let g = gen(include_str!("c0.ungram"), get_kind);
  write_rust_file("src/kind.rs", &g.kind).unwrap();
  write_rust_file("src/ast.rs", &g.ast).unwrap();
}
