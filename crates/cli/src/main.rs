//! A CLI wrapper over `analysis`.

use analysis::{Db, Uri};
use gumdrop::Options;
use std::collections::HashMap;

#[derive(Debug, Options)]
struct Config {
  #[options(help = "print this help")]
  pub help: bool,
  #[options(free, help = "source files")]
  pub source: Vec<String>,
}

fn read_file(s: &str) -> Option<String> {
  match std::fs::read_to_string(s) {
    Ok(x) => Some(x),
    Err(e) => {
      eprintln!("{}: {}", s, e);
      None
    }
  }
}

fn run(conf: Config) -> Option<bool> {
  let mut files = HashMap::new();
  for file in conf.source {
    let contents = read_file(&file)?;
    files.insert(Uri::new(file.into()), contents);
  }
  let ide = Db::new(files);
  let diagnostics = ide.all_diagnostics();
  for &(uri, ref ds) in diagnostics.iter() {
    for d in ds.iter() {
      eprintln!("{}:{}", uri.as_path().display(), d);
    }
  }
  Some(diagnostics.iter().all(|&(_, ref ds)| ds.is_empty()))
}

const BIG_STACK_SIZE: usize = 180 * 1024 * 1024;

fn main() {
  let conf = Config::parse_args_default_or_exit();
  let ec = match std::thread::Builder::new()
    .name("run".to_owned())
    .stack_size(BIG_STACK_SIZE)
    .spawn(|| run(conf))
    .expect("couldn't spawn run")
    .join()
  {
    Ok(Some(true)) => {
      eprintln!("no errors");
      0
    }
    Ok(Some(false)) => 1,
    Ok(None) => 2,
    Err(_) => 3,
  };
  std::process::exit(ec)
}
