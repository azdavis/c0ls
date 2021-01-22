use crate::string::pascal_to_snake;
use crate::util::{ident, unwrap_node, Cx};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use ungrammar::{Node, Rule, Token};

pub(crate) fn get(cx: &Cx, name: Ident, rules: &[Rule]) -> TokenStream {
  let fields = rules.iter().map(|rule| field(cx, rule));
  quote! {
    pub struct #name(SyntaxNode);
    impl #name {
      pub fn cast(node: SyntaxNode) -> Option<Self> {
        if node.kind() == SyntaxKind::#name {
          Some(Self(node))
        } else {
          None
        }
      }
      pub fn syntax(&self) -> &SyntaxNode {
        &self.0
      }
      #(#fields)*
    }
  }
}

enum Modifier {
  Rep,
  Opt,
  Regular,
}

fn field(cx: &Cx, rule: &Rule) -> TokenStream {
  match rule {
    Rule::Labeled { label, rule } => labeled_field(cx, label.as_str(), rule),
    Rule::Node(node) => node_field(cx, Modifier::Regular, None, *node),
    Rule::Token(tok) => token_field(cx, None, *tok),
    Rule::Opt(r) => node_field(cx, Modifier::Opt, None, unwrap_node(r)),
    Rule::Rep(r) => node_field(cx, Modifier::Rep, None, unwrap_node(r)),
    Rule::Alt(_) | Rule::Seq(_) => unreachable!(),
  }
}

fn token_field(cx: &Cx, name: Option<&str>, token: Token) -> TokenStream {
  let kind = cx.tokens.name(token).unwrap();
  let name = match name {
    None => ident(&pascal_to_snake(kind)),
    Some(x) => ident(x),
  };
  let kind = ident(kind);
  quote! {
    pub fn #name(&self) -> Option<SyntaxToken> {
      child_token(self.syntax(), SyntaxKind::#kind)
    }
  }
}

fn labeled_field(cx: &Cx, label: &str, rule: &Rule) -> TokenStream {
  match rule {
    Rule::Node(node) => node_field(cx, Modifier::Regular, Some(label), *node),
    Rule::Token(tok) => token_field(cx, Some(label), *tok),
    Rule::Opt(r) => node_field(cx, Modifier::Opt, Some(label), unwrap_node(r)),
    Rule::Labeled { .. } | Rule::Seq(_) | Rule::Alt(_) | Rule::Rep(_) => {
      unreachable!()
    }
  }
}

fn node_field(
  cx: &Cx,
  modifier: Modifier,
  name: Option<&str>,
  node: Node,
) -> TokenStream {
  let kind = &cx.grammar[node].name;
  let owned;
  let name = match name {
    None => {
      owned = pascal_to_snake(kind);
      &owned
    }
    Some(x) => x,
  };
  let kind = ident(kind);
  let name_ident;
  let ret_ty;
  let body;
  match modifier {
    Modifier::Rep => {
      name_ident = format_ident!("{}s", name);
      ret_ty = quote! { impl Iterator<Item = #kind> };
      body = quote! {
        self.syntax().children().filter_map(#kind::cast)
      };
    }
    Modifier::Opt | Modifier::Regular => {
      name_ident = ident(name);
      ret_ty = quote! { Option<#kind> };
      body = quote! {
        self.syntax().children().find_map(#kind::cast)
      };
    }
  }
  quote! {
    pub fn #name_ident(&self) -> #ret_ty {
      #body
    }
  }
}
