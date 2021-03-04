use identifier_case::snake_to_pascal;
use syntax_gen::{gen, TokenKind};

const SPECIAL: [(&str, &str); 6] = [
  ("Ident", "an identifier"),
  ("DecLit", "an integer literal"),
  ("HexLit", "a hexadecimal integer literal"),
  ("StringLit", "a string literal"),
  ("CharLit", "a char literal"),
  ("Pragma", "a pragma"),
];

fn get_token(name: &str) -> (TokenKind, String) {
  if let Some(desc) = SPECIAL.iter().find_map(|&(n, d)| (n == name).then(|| d))
  {
    (TokenKind::Special(desc), name.to_owned())
  } else if name == "->" {
    (TokenKind::Punctuation, "Arrow".to_owned())
  } else if name.chars().any(|c| c.is_ascii_alphabetic()) {
    let mut ret = snake_to_pascal(name);
    ret.push_str("Kw");
    (TokenKind::Keyword, ret)
  } else {
    let mut ret = String::new();
    for c in name.chars() {
      ret.push_str(char_name::get(c));
    }
    (TokenKind::Punctuation, ret)
  }
}

fn main() {
  gen(
    "C0",
    &["Whitespace", "LineComment", "BlockComment", "Invalid"],
    include_str!("c0.ungram").parse().unwrap(),
    get_token,
  )
  .unwrap();
}
