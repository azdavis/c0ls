use std::fs::OpenOptions;
use std::io::Write as _;
use std::process::{Command, Stdio};
use syntax_gen::gen;

fn write_rust_file(name: &str, contents: &str) {
  let mut proc = Command::new("rustfmt")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();
  proc
    .stdin
    .take()
    .unwrap()
    .write_all(contents.as_bytes())
    .unwrap();
  assert!(proc.wait().unwrap().success());
  let mut stdout = proc.stdout.take().unwrap();
  let mut out_file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(name)
    .unwrap();
  std::io::copy(&mut stdout, &mut out_file).unwrap();
}

fn main() {
  let g = gen();
  write_rust_file("src/kind.rs", &g.kind);
  write_rust_file("src/ast.rs", &g.ast);
}
