use crate::util::{ident, unwrap_node, unwrap_token, Cx};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use ungrammar::Rule;

pub(crate) fn get(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  match rules.first().unwrap() {
    Rule::Node(_) => get_nodes(cx, name, rules),
    Rule::Token(_) => get_tokens(cx, name, rules),
    bad => panic!("bad alt rule {:?}", bad),
  }
}

fn get_nodes(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  let (defs, arms): (Vec<_>, Vec<_>) = rules
    .iter()
    .map(|rule| {
      let name = ident(&cx.grammar[unwrap_node(rule)].name);
      let def = quote! { #name(#name) };
      let cast = quote! { SK::#name => Self::#name(#name(node)) };
      let syntax = quote! { Self::#name(x) => x.syntax() };
      (def, (cast, syntax))
    })
    .unzip();
  let (casts, syntaxes): (Vec<_>, Vec<_>) = arms.into_iter().unzip();
  quote! {
    pub enum #name {
      #(#defs ,)*
    }
    impl Cast for #name {
      fn cast(elem: SyntaxElement) -> Option<Self> {
        let node = elem.into_node()?;
        let ret = match node.kind() {
          #(#casts ,)*
          _ => return None,
        };
        Some(ret)
      }
    }
    impl Syntax for #name {
      fn syntax(&self) -> &SyntaxNode {
        match self {
          #(#syntaxes ,)*
        }
      }
    }
  }
}

fn get_tokens(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  let name_kind = format_ident!("{}Kind", name);
  let (defs, casts): (Vec<_>, Vec<_>) = rules
    .iter()
    .map(|rule| {
      let name = ident(&cx.tokens.name(unwrap_token(rule)));
      let def = quote! { #name };
      let cast = quote! { SK::#name => #name_kind::#name };
      (def, cast)
    })
    .unzip();
  quote! {
    pub enum #name_kind {
      #(#defs ,)*
    }
    pub struct #name {
      pub token: SyntaxToken,
      pub kind: #name_kind,
    }
    impl Cast for #name {
      fn cast(elem: SyntaxElement) -> Option<Self> {
        let token = elem.into_token()?;
        let kind = match token.kind() {
          #(#casts ,)*
          _ => return None,
        };
        Some(Self { kind, token })
      }
    }
  }
}
