use identifier_case::snake_to_pascal;
use syntax_gen::{gen, Token, TokenKind};

const SPECIAL: [(&str, &str); 6] = [
  ("Ident", "an identifier"),
  ("DecLit", "an integer literal"),
  ("HexLit", "a hexadecimal integer literal"),
  ("StringLit", "a string literal"),
  ("CharLit", "a char literal"),
  ("Pragma", "a pragma"),
];

fn get_token(name: &str) -> (TokenKind, Token) {
  let (kind, name, desc) = if let Some(desc) =
    SPECIAL.iter().find_map(|&(n, d)| (n == name).then_some(d))
  {
    (TokenKind::Special, name.to_owned(), Some(desc.to_owned()))
  } else if name == "->" {
    (TokenKind::Punctuation, "Arrow".to_owned(), None)
  } else if name.chars().any(|c| c.is_ascii_alphabetic()) {
    let mut ret = snake_to_pascal(name);
    ret.push_str("Kw");
    (TokenKind::Keyword, ret, None)
  } else {
    let mut ret = String::new();
    for c in name.chars() {
      ret.push_str(char_name::get(c));
    }
    (TokenKind::Punctuation, ret, None)
  };
  let tok = Token {
    name,
    desc,
    doc: None,
  };
  (kind, tok)
}

fn main() {
  let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR should be set");
  gen(
    std::path::Path::new(out_dir.as_str()),
    "C0",
    &["Whitespace", "LineComment", "BlockComment", "Invalid"],
    include_str!("c0.ungram").parse().unwrap(),
    get_token,
  )
  .unwrap();
}
