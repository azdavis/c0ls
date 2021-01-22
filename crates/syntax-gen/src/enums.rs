use crate::util::{ident, Cx};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use ungrammar::Rule;

pub(crate) fn get(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  let (defs, casts): (Vec<_>, Vec<_>) =
    rules.iter().map(|rule| variant(cx, rule)).unzip();
  quote! {
    pub enum #name {
      #(#defs ,)*
    }
    impl Cast for #name {
      fn cast(node: SyntaxNode) -> Option<Self> {
        match node.kind() {
          #(#casts ,)*
          _ => None,
        }
      }
    }
  }
}

fn variant(cx: &Cx, rule: &Rule) -> (TokenStream, TokenStream) {
  let name;
  let def;
  let cast;
  match rule {
    Rule::Node(node) => {
      name = ident(&cx.grammar[*node].name);
      def = quote! { #name(#name) };
      cast = quote! {
        SK::#name => Some(Self::#name(#name(node)))
      };
    }
    Rule::Token(tok) => {
      name = ident(cx.tokens.name(*tok));
      def = quote! { #name(SyntaxToken) };
      cast = quote! {
        SK::#name => Some(Self::#name(node.first_token().unwrap()))
      };
    }
    Rule::Labeled { .. }
    | Rule::Seq(_)
    | Rule::Alt(_)
    | Rule::Opt(_)
    | Rule::Rep(_) => unreachable!(),
  }
  (def, cast)
}
