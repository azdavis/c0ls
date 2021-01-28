use crate::tokens::Tokens;
use proc_macro2::{Ident, Literal};
use quote::format_ident;
use rustc_hash::FxHashMap;
use std::cmp::Reverse;
use ungrammar::{Grammar, Node, Rule, Token};

#[derive(Debug)]
pub(crate) struct Cx {
  pub(crate) grammar: Grammar,
  pub(crate) tokens: Tokens,
}

pub(crate) fn ident(s: &str) -> Ident {
  format_ident!("{}", s)
}

pub(crate) fn unwrap_node(rule: &Rule) -> Node {
  match rule {
    Rule::Node(node) => *node,
    _ => panic!("unwrap_node on {:?}", rule),
  }
}

pub(crate) fn unwrap_token(rule: &Rule) -> Token {
  match rule {
    Rule::Token(tok) => *tok,
    _ => panic!("unwrap_token on {:?}", rule),
  }
}

pub(crate) fn sort_tokens(
  grammar: &Grammar,
  m: FxHashMap<Token, String>,
) -> impl Iterator<Item = (Literal, Ident)> + '_ {
  let mut xs: Vec<_> = m
    .into_iter()
    .map(|(tok, s)| (grammar[tok].name.as_bytes(), s))
    .collect();
  xs.sort_unstable_by_key(|&(name, _)| (Reverse(name.len()), name));
  xs.into_iter()
    .map(|(bs, s)| (Literal::byte_string(bs), ident(&s)))
}
