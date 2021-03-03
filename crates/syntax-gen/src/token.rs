use identifier_case::snake_to_pascal;
use rustc_hash::FxHashMap;
use ungrammar::{Grammar, Token};

#[derive(Debug)]
pub(crate) struct TokenDb {
  pub(crate) punctuation: FxHashMap<Token, String>,
  pub(crate) keywords: FxHashMap<Token, String>,
  pub(crate) special: FxHashMap<Token, (String, &'static str)>,
}

/// What kind of token this is.
pub enum TokenKind {
  /// Punctuation, like `{` or `}` or `++`
  Punctuation,
  /// Keywords, i.e. they might be confused as identifiers.
  Keyword,
  /// Special tokens, with a given description.
  Special(&'static str),
}

fn get_kind(name: &str) -> (TokenKind, String) {
  if let Some(desc) = CONTENT
    .iter()
    .find_map(|&(n, desc)| (n == name).then(|| desc))
  {
    (TokenKind::Special(desc), name.to_owned())
  } else if name == "->" {
    (TokenKind::Punctuation, "Arrow".to_owned())
  } else if name.chars().any(|c| c.is_ascii_alphabetic()) {
    let mut ins = snake_to_pascal(name);
    ins.push_str("Kw");
    (TokenKind::Keyword, ins)
  } else {
    let mut ins = String::new();
    for c in name.chars() {
      let s = match char_name::get(c) {
        Some(x) => x,
        None => panic!("don't know the name for {}", c),
      };
      ins.push_str(s);
    }
    (TokenKind::Punctuation, ins)
  }
}

impl TokenDb {
  pub(crate) fn new(grammar: &Grammar) -> Self {
    let mut punctuation = FxHashMap::default();
    let mut keywords = FxHashMap::default();
    let mut special = FxHashMap::default();
    for token in grammar.tokens() {
      let (kind, name) = get_kind(grammar[token].name.as_ref());
      match kind {
        TokenKind::Punctuation => {
          assert!(punctuation.insert(token, name).is_none());
        }
        TokenKind::Keyword => {
          assert!(keywords.insert(token, name).is_none());
        }
        TokenKind::Special(desc) => {
          assert!(special.insert(token, (name, desc)).is_none());
        }
      }
    }
    Self {
      punctuation,
      keywords,
      special,
    }
  }

  pub(crate) fn name(&self, token: Token) -> &str {
    if let Some(x) = self.punctuation.get(&token) {
      x.as_ref()
    } else if let Some(x) = self.keywords.get(&token) {
      x.as_ref()
    } else if let Some(&(ref x, _)) = self.special.get(&token) {
      x.as_ref()
    } else {
      panic!("{:?} does not have a name", token)
    }
  }
}

const CONTENT: [(&str, &str); 6] = [
  ("Ident", "an identifier"),
  ("DecLit", "an integer literal"),
  ("HexLit", "a hexadecimal integer literal"),
  ("StringLit", "a string literal"),
  ("CharLit", "a char literal"),
  ("Pragma", "a pragma"),
];
