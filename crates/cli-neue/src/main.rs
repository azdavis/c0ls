//! A hacky CLI wrapper over `analysis`.

use analysis::{Db, Uri};
use gumdrop::Options;
use std::collections::HashMap;

#[derive(Debug, Options)]
struct Config {
  #[options(help = "print this help")]
  pub help: bool,
  #[options(short = "l", help = "header file")]
  pub header: Option<String>,
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

fn get_files(files: Vec<String>) -> Option<HashMap<Uri, String>> {
  let mut ret = HashMap::new();
  for file in files {
    let contents = read_file(&file)?;
    ret.insert(Uri::new(file.into()), contents);
  }
  Some(ret)
}

fn run(conf: Config) -> Option<bool> {
  let header = match conf.header {
    None => None,
    Some(header) => {
      let contents = read_file(&header)?;
      Some((Uri::new(header.into()), contents))
    }
  };
  let files = get_files(conf.source)?;
  let ide = Db::new(files, header);
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
    .expect("couldn't spawn")
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
