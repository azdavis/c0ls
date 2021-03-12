use syntax::ast::Root;

fn check(inp: &str, out: &str) {
  let inp_root = get_root(inp);
  let out_root = get_root(out);
  assert_eq!(crate::get(inp_root).unwrap(), out);
  // idempotent
  assert_eq!(crate::get(out_root).unwrap(), out);
}

fn get_root(s: &str) -> Root {
  let lexed = lex::get(s);
  let parsed = parse::get(&lexed.tokens);
  assert!(lexed.errors.is_empty());
  assert!(parsed.errors.is_empty());
  parsed.root
}

#[test]
fn simple() {
  check(
    include_str!("data/simple.inp.c0"),
    include_str!("data/simple.out.c0"),
  );
}

#[test]
fn if_return() {
  check(
    include_str!("data/if_return.inp.c0"),
    include_str!("data/if_return.out.c0"),
  );
}
