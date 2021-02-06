use identifier_case::snake_to_pascal;
use rustc_hash::FxHashMap;
use ungrammar::{Grammar, Token};

#[derive(Debug)]
pub(crate) struct TokenDb {
  pub(crate) punctuation: FxHashMap<Token, String>,
  pub(crate) keywords: FxHashMap<Token, String>,
  pub(crate) special: FxHashMap<Token, String>,
}

impl TokenDb {
  pub(crate) fn name(&self, token: Token) -> &str {
    if let Some(x) = self.punctuation.get(&token) {
      return x;
    }
    if let Some(x) = self.keywords.get(&token) {
      return x;
    }
    if let Some(x) = self.special.get(&token) {
      return x;
    }
    panic!("{:?} does not have a name", token)
  }
}

pub const CONTENT: [(&str, &str); 6] = [
  ("Ident", "an identifier"),
  ("DecLit", "an integer literal"),
  ("HexLit", "a hexadecimal integer literal"),
  ("StringLit", "a string literal"),
  ("CharLit", "a char literal"),
  ("Pragma", "a pragma"),
];

pub(crate) fn get(grammar: &Grammar) -> TokenDb {
  let mut punctuation = FxHashMap::default();
  let mut keywords = FxHashMap::default();
  let mut special = FxHashMap::default();
  for token in grammar.tokens() {
    let name = &grammar[token].name;
    let (map, ins) = if CONTENT.iter().any(|&(n, _)| n == name) {
      (&mut special, name.to_owned())
    } else if name == "->" {
      (&mut punctuation, "Arrow".to_owned())
    } else if name.chars().any(|c| c.is_ascii_alphabetic()) {
      let mut ins = snake_to_pascal(name);
      ins.push_str("Kw");
      (&mut keywords, ins)
    } else {
      let mut ins = String::new();
      for c in name.chars() {
        ins.push_str(char_name(c));
      }
      (&mut punctuation, ins)
    };
    assert!(map.insert(token, ins).is_none());
  }
  assert_eq!(CONTENT.len(), special.len());
  TokenDb {
    punctuation,
    keywords,
    special,
  }
}

fn char_name(c: char) -> &'static str {
  match c {
    '-' => "Minus",
    ',' => "Comma",
    ';' => "Semicolon",
    ':' => "Colon",
    '!' => "Bang",
    '?' => "Question",
    '.' => "Dot",
    '(' => "LRound",
    ')' => "RRound",
    '[' => "LSquare",
    ']' => "RSquare",
    '{' => "LCurly",
    '}' => "RCurly",
    '*' => "Star",
    '/' => "Slash",
    '&' => "And",
    '%' => "Percent",
    '^' => "Carat",
    '+' => "Plus",
    '<' => "Lt",
    '=' => "Eq",
    '>' => "Gt",
    '|' => "Bar",
    '~' => "Tilde",
    _ => panic!("don't know the name for {}", c),
  }
}
