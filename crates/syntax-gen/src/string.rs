pub(crate) fn snake_to_pascal(s: &str) -> String {
  let mut ret = String::with_capacity(s.len());
  let mut is_cap = true;
  for c in s.chars() {
    if c == '_' {
      is_cap = true;
    } else if is_cap {
      is_cap = false;
      ret.push(c.to_ascii_uppercase());
    } else {
      ret.push(c.to_ascii_lowercase());
    }
  }
  ret
}

/// also works for camelCase
pub(crate) fn pascal_to_snake(s: &str) -> String {
  let mut ret = String::with_capacity(s.len());
  let mut cs = s.chars();
  // don't put a _ at the start
  if let Some(c) = cs.next() {
    ret.push(c.to_ascii_lowercase());
  }
  for c in cs {
    if c.is_ascii_uppercase() {
      ret.push('_');
    }
    ret.push(c.to_ascii_lowercase());
  }
  ret
}

pub(crate) fn char_name(c: char) -> &'static str {
  match c {
    '-' => "Minus",
    ',' => "Comma",
    ';' => "Semicolon",
    ':' => "Colon",
    '!' => "Bang",
    '?' => "Question",
    '.' => "Dot",
    '(' => "LRound",
    ')' => "RRound",
    '[' => "LSquare",
    ']' => "RSquare",
    '{' => "LCurly",
    '}' => "RCurly",
    '*' => "Star",
    '/' => "Slash",
    '&' => "And",
    '%' => "Percent",
    '^' => "Carat",
    '+' => "Plus",
    '<' => "Lt",
    '=' => "Eq",
    '>' => "Gt",
    '|' => "Bar",
    '~' => "Tilde",
    _ => unreachable!("don't know the name for {}", c),
  }
}

#[test]
fn snake_to_pascal_t() {
  assert_eq!(snake_to_pascal("fella"), "Fella");
  assert_eq!(snake_to_pascal("the_best"), "TheBest");
  assert_eq!(snake_to_pascal("HEY_THERE_DUDE"), "HeyThereDude");
}
