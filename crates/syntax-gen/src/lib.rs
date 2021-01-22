#![deny(rust_2018_idioms)]

mod enums;
mod string;
mod structs;
mod tokens;
mod util;

use crate::util::{ident, sort_tokens, Cx};
use quote::quote;
use ungrammar::{Grammar, Rule};

pub fn gen() -> String {
  let grammar: Grammar = include_str!("c0.ungram").parse().unwrap();
  let tokens = tokens::get(&grammar);
  let cx = Cx { grammar, tokens };
  let mut types = Vec::new();
  let mut syntax_kinds = Vec::new();
  for node in cx.grammar.iter() {
    let data = &cx.grammar[node];
    let name = ident(&data.name);
    let ty = match &data.rule {
      Rule::Seq(rules) => structs::get(&cx, name.clone(), rules),
      Rule::Alt(rules) => enums::get(&cx, name.clone(), rules),
      rule => structs::get(&cx, name.clone(), std::slice::from_ref(rule)),
    };
    types.push(ty);
    syntax_kinds.push(name);
  }
  let Cx { grammar, tokens } = cx;
  syntax_kinds.extend(sort_tokens(&grammar, tokens.content).map(|x| x.1));
  let keywords: Vec<_> = sort_tokens(&grammar, tokens.keywords).collect();
  let keyword_arms = keywords
    .iter()
    .map(|(bs, kind)| quote! { #bs => Some(Self::#kind) });
  syntax_kinds.extend(keywords.iter().map(|x| x.1.clone()));
  let punctuation: Vec<_> = sort_tokens(&grammar, tokens.punctuation).collect();
  let punctuation_len = punctuation.len();
  let punctuation_elements = punctuation
    .iter()
    .map(|(bs, kind)| quote! { (#bs, Self::#kind) });
  syntax_kinds.extend(punctuation.iter().map(|x| x.1.clone()));
  let last = syntax_kinds.last().unwrap();
  let ret = quote! {
    pub use event_parse;
    pub use rowan;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(u16)]
    pub enum SyntaxKind {
      Whitespace,
      LineComment,
      BlockComment,
      Invalid,
      UseKw,
      #(#syntax_kinds ,)*
    }

    impl SyntaxKind {
      pub const PUNCTUATION: [(&'static [u8], Self); #punctuation_len] = [
        #(#punctuation_elements ,)*
      ];

      pub fn keyword(bs: &[u8]) -> Option<Self> {
        match bs {
          #(#keyword_arms ,)*
          _ => None,
        }
      }
    }

    impl event_parse::Triviable for SyntaxKind {
      fn is_trivia(&self) -> bool {
        matches!(
          *self,
          Self::Whitespace
          | Self::LineComment
          | Self::BlockComment
          | Self::Invalid
        )
      }
    }

    impl From<SyntaxKind> for rowan::SyntaxKind {
      fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
      }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum C0 {}

    impl rowan::Language for C0 {
      type Kind = SyntaxKind;

      fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::#last as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
      }

      fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
      }
    }

    pub type SyntaxNode = rowan::SyntaxNode<C0>;
    pub type SyntaxToken = rowan::SyntaxToken<C0>;

    pub mod ast {
      use super::{SyntaxKind, SyntaxNode, SyntaxToken};

      pub trait Cast: Sized {
        fn cast(node: SyntaxNode) -> Option<Self>;
      }

      pub trait Syntax {
        fn syntax(&self) -> &SyntaxNode;
      }

      #[inline]
      fn child_token(
        parent: &SyntaxNode,
        kind: SyntaxKind,
      ) -> Option<SyntaxToken> {
        parent
          .children_with_tokens()
          .filter_map(rowan::NodeOrToken::into_token)
          .find(|tok| tok.kind() == kind)
      }

      #(#types)*
    }
  };
  ret.to_string()
}
