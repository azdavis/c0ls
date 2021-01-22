use crate::string::{char_name, snake_to_pascal};
use std::collections::HashMap;
use ungrammar::{Grammar, Token};

const CONTENT_SIGIL: char = '$';

#[derive(Debug, Default)]
pub(crate) struct Tokens {
  pub(crate) punctuation: HashMap<Token, String>,
  pub(crate) keywords: HashMap<Token, String>,
  pub(crate) content: HashMap<Token, String>,
  use_token: Option<Token>,
}

impl Tokens {
  pub(crate) fn name(&self, token: Token) -> Option<&str> {
    if let Some(x) = self.punctuation.get(&token) {
      return Some(x);
    }
    if let Some(x) = self.keywords.get(&token) {
      return Some(x);
    }
    if let Some(x) = self.content.get(&token) {
      return Some(x);
    }
    if Some(token) == self.use_token {
      return Some("UseKw");
    }
    None
  }
}

pub(crate) fn get(grammar: &Grammar) -> Tokens {
  let mut ret = Tokens::default();
  for token in grammar.tokens() {
    let name = &grammar[token].name;
    let mut cs = name.chars();
    if name == "#use" {
      ret.use_token = Some(token);
      continue;
    }
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
