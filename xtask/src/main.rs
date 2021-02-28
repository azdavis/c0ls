//! A task runner using the [xtask spec][1].
//!
//! [1]: https://github.com/matklad/cargo-xtask

use anyhow::{bail, Result};
use pico_args::Arguments;
use std::path::Path;
use walkdir::WalkDir;
use xshell::{cmd, cp, mkdir_p, pushd};

#[inline]
fn show_help() {
  print!("{}", include_str!("help.txt"));
}

fn finish_args(args: Arguments) -> Result<()> {
  let args = args.finish();
  if !args.is_empty() {
    bail!("unused arguments: {:?}", args);
  }
  Ok(())
}

fn ck_test_data() -> Result<()> {
  for &cr in ["analysis", "fmt"].iter() {
    let tests = format!("crates/{}/src/tests", cr);
    for entry in WalkDir::new(&format!("{}/data", tests)) {
      let entry = entry?;
      let name = entry.path().file_name().unwrap();
      cmd!("git grep -q {name} -- {tests}/mod.rs").run()?;
    }
  }
  Ok(())
}

fn run() -> Result<()> {
  let mut args = Arguments::from_env();
  if args.contains(["-h", "--help"]) {
    show_help();
    return Ok(());
  }
  let subcommand = match args.subcommand()? {
    Some(x) => x,
    None => {
      show_help();
      return Ok(());
    }
  };
  let _d = pushd(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap())?;
  match subcommand.as_str() {
    "ci" => {
      finish_args(args)?;
      ck_test_data()?;
      cmd!("cargo clippy").run()?;
      cmd!("cargo test").run()?;
    }
    "ck-test-data" => {
      finish_args(args)?;
      ck_test_data()?
    }
    "mk-vscode-ext" => {
      finish_args(args)?;
      cmd!("cargo build -p c0ls").run()?;
      mkdir_p("extensions/vscode/out")?;
      cp("target/debug/c0ls", "extensions/vscode/out/c0ls")?;
      let _d = pushd("extensions/vscode")?;
      if std::fs::metadata("node_modules").is_err() {
        cmd!("npm install").run()?;
      }
      cmd!("npm run build").run()?;
    }
    s => bail!("unknown subcommand: {}", s),
  }
  Ok(())
}

fn main() {
  match run() {
    Ok(()) => {}
    Err(e) => {
      eprintln!("{}", e);
      std::process::exit(1);
    }
  }
}
