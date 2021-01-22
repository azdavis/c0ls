use crate::tokens::Tokens;
use proc_macro2::Ident;
use quote::format_ident;
use ungrammar::{Grammar, Node, Rule};

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
    _ => unreachable!(),
  }
}
