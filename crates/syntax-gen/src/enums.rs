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
    impl #name {
      pub fn cast(node: SyntaxNode) -> Option<Self> {
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
        SyntaxKind::#name => Some(Self::#name(#name(node)))
      };
    }
    Rule::Token(tok) => {
      name = ident(cx.tokens.name(*tok).unwrap());
      def = quote! { #name(SyntaxToken) };
      cast = quote! {
        SyntaxKind::#name => Some(Self::#name(node.first_token().unwrap()))
      };
    }
    Rule::Labeled { .. } => panic!("Labeled"),
    Rule::Seq(_) => panic!("Seq"),
    Rule::Alt(_) => panic!("Alt"),
    Rule::Opt(_) => panic!("Opt"),
    Rule::Rep(_) => panic!("Rep"),
  }
  (def, cast)
}
