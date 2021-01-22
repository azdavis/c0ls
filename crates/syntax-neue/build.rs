use std::fs::OpenOptions;
use std::io::Write as _;
use std::process::{Command, Stdio};
use syntax_gen::gen;

const OUT_FILE: &str = "src/generated.rs";

fn main() {
  let mut proc = Command::new("rustfmt")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();
  proc
    .stdin
    .take()
    .unwrap()
    .write_all(gen().as_bytes())
    .unwrap();
  assert!(proc.wait().unwrap().success());
  let mut stdout = proc.stdout.take().unwrap();
  let mut out_file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(OUT_FILE)
    .unwrap();
  std::io::copy(&mut stdout, &mut out_file).unwrap();
}
