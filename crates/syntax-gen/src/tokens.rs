//! TODO change this to just use the tokens iterator once the changes are
//! merged

use crate::slow_map::SlowMap;
use crate::string::{char_name, snake_to_pascal};
use ungrammar::{Grammar, Rule, Token};

#[derive(Debug, Default)]
pub(crate) struct Tokens {
  store: SlowMap<Token, String>,
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
  for node in grammar.iter() {
    do_rule(&mut ret, grammar, &grammar[node].rule)
  }
  ret
}

fn do_rules(ts: &mut Tokens, grammar: &Grammar, rules: &[Rule]) {
  for rule in rules {
    do_rule(ts, grammar, rule)
  }
}

fn do_rule(ts: &mut Tokens, grammar: &Grammar, rule: &Rule) {
  match rule {
    Rule::Token(tok) => add_token(ts, &grammar, *tok),
    Rule::Seq(rules) | Rule::Alt(rules) => do_rules(ts, grammar, rules),
    Rule::Labeled { rule, .. } | Rule::Opt(rule) | Rule::Rep(rule) => {
      do_rule(ts, grammar, rule)
    }
    // no need to recurse since we're iterating over all nodes.
    Rule::Node(_) => {}
  }
}

fn add_token(ts: &mut Tokens, grammar: &Grammar, token: Token) {
  if ts.name(token).is_some() {
    return;
  }
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
  assert!(ts.store.insert(token, ins).is_none());
}
