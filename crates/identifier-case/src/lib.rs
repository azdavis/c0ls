//! Utilities for working with identifiers in different cases.

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]

/// Works with either upper or lower snake case.
pub fn snake_to_pascal(s: &str) -> String {
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

/// Also works for camelCase.
pub fn pascal_to_snake(s: &str) -> String {
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

#[test]
fn snake_to_pascal_t() {
  assert_eq!(snake_to_pascal("fella"), "Fella");
  assert_eq!(snake_to_pascal("the_best"), "TheBest");
  assert_eq!(snake_to_pascal("HEY_THERE_DUDE"), "HeyThereDude");
}

#[test]
fn pascal_to_snake_t() {
  assert_eq!(pascal_to_snake("FooBar"), "foo_bar");
  assert_eq!(pascal_to_snake("readFile"), "read_file");
  assert_eq!(pascal_to_snake("GetLine"), "get_line");
}
