use identifier_case::snake_to_pascal;
use rustc_hash::FxHashMap;
use ungrammar::{Grammar, Token};

const CONTENT_SIGIL: char = '$';

#[derive(Debug, Default)]
pub(crate) struct Tokens {
  pub(crate) punctuation: FxHashMap<Token, String>,
  pub(crate) keywords: FxHashMap<Token, String>,
  pub(crate) content: FxHashMap<Token, String>,
  use_token: Option<Token>,
}

impl Tokens {
  pub(crate) fn name(&self, token: Token) -> &str {
    if let Some(x) = self.punctuation.get(&token) {
      return x;
    }
    if let Some(x) = self.keywords.get(&token) {
      return x;
    }
    if let Some(x) = self.content.get(&token) {
      return x;
    }
    if Some(token) == self.use_token {
      return "UseKw";
    }
    panic!("{:?} does not have a name", token)
  }
}

pub(crate) fn get(grammar: &Grammar) -> Tokens {
  let mut ret = Tokens::default();
  for token in grammar.tokens() {
    let name = &grammar[token].name;
    if name == "#use" {
      ret.use_token = Some(token);
      continue;
    }
    let mut cs = name.chars();
    let (map, ins) = if cs.next().unwrap() == CONTENT_SIGIL {
      (&mut ret.content, cs.as_str().to_owned())
    } else if name == "->" {
      (&mut ret.punctuation, "Arrow".to_owned())
    } else if name.chars().any(|c| c.is_ascii_alphabetic()) {
      let mut ins = snake_to_pascal(name);
      ins.push_str("Kw");
      (&mut ret.keywords, ins)
    } else {
      let mut ins = String::new();
      for c in name.chars() {
        ins.push_str(char_name(c));
      }
      (&mut ret.punctuation, ins)
    };
    assert!(map.insert(token, ins).is_none());
  }
  ret
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
