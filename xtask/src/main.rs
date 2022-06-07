//! A task runner using the [xtask spec][1].
//!
//! [1]: https://github.com/matklad/cargo-xtask

use anyhow::{bail, Result};
use pico_args::Arguments;
use std::path::Path;
use walkdir::WalkDir;
use xshell::{cmd, Shell};

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

fn ck_test_data(sh: &Shell) -> Result<()> {
  for &cr in ["analysis", "fmt"].iter() {
    let tests = format!("crates/{}/src/tests", cr);
    for entry in WalkDir::new(&format!("{}/data", tests)) {
      let entry = entry?;
      let name = entry.path().file_name().unwrap();
      cmd!(sh, "git grep -q {name} -- {tests}/mod.rs").run()?;
    }
  }
  Ok(())
}

fn main() -> Result<()> {
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
  let sh = Shell::new()?;
  let _d = sh.push_dir(Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap());
  match subcommand.as_str() {
    "ci" => {
      finish_args(args)?;
      ck_test_data(&sh)?;
      // run this first to generate code
      cmd!(sh, "cargo test --no-run").run()?;
      cmd!(sh, "cargo fmt -- --check").run()?;
      cmd!(sh, "cargo clippy").run()?;
      cmd!(sh, "cargo test").run()?;
    }
    "ck-test-data" => {
      finish_args(args)?;
      ck_test_data(&sh)?
    }
    "mk-vscode-ext" => {
      finish_args(args)?;
      cmd!(sh, "cargo build -p c0ls").run()?;
      sh.create_dir("extensions/vscode/out")?;
      sh.copy_file("target/debug/c0ls", "extensions/vscode/out/c0ls")?;
      let _d = sh.push_dir("extensions/vscode");
      if std::fs::metadata("node_modules").is_err() {
        cmd!(sh, "npm install").run()?;
      }
      cmd!(sh, "npm run build").run()?;
    }
    s => bail!("unknown subcommand: {}", s),
  }
  Ok(())
}
