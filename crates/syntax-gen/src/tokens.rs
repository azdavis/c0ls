use crate::string::{char_name, snake_to_pascal};
use std::collections::HashMap;
use ungrammar::{Grammar, Token};

#[derive(Debug, Default)]
pub(crate) struct Tokens {
  store: HashMap<Token, String>,
  use_token: Option<Token>,
}

const CONTENT_SIGIL: char = '$';

impl Tokens {
  pub(crate) fn name(&self, token: Token) -> Option<&str> {
    self.store.get(&token).map(AsRef::as_ref)
  }

  pub(crate) fn all_names(&self) -> impl Iterator<Item = &str> {
    self.store.values().map(AsRef::as_ref)
  }
}

pub(crate) fn get(grammar: &Grammar) -> Tokens {
  let mut ret = Tokens::default();
  for token in grammar.tokens() {
    let name = &grammar[token].name;
    let mut cs = name.chars();
    let ins = if cs.next().unwrap() == CONTENT_SIGIL {
      cs.as_str().to_owned()
    } else if name == "->" {
      "Arrow".to_owned()
    } else if name == "#use" {
      "UseKw".to_owned()
    } else if name.chars().any(|c| c.is_ascii_alphabetic()) {
      let mut ins = snake_to_pascal(name);
      ins.push_str("Kw");
      ins
    } else {
      let mut ins = String::new();
      for c in name.chars() {
        ins.push_str(char_name(c));
      }
      ins
    };
    assert!(ret.store.insert(token, ins).is_none());
  }
  ret
}
