//! A CLI wrapper over `analysis`.

use analysis::{url::Url, Db};
use gumdrop::Options;
use rustc_hash::FxHashMap;
use std::path::Path;

#[derive(Debug, Options)]
struct Config {
  #[options(help = "print this help")]
  pub help: bool,
  #[options(help = "format each source file")]
  pub format: bool,
  #[options(help = "show the CST of each source file")]
  pub cst: bool,
  #[options(free, help = "source files")]
  pub source: Vec<String>,
}

fn read_file(s: &str) -> Option<String> {
  match std::fs::read_to_string(s) {
    Ok(x) => Some(x),
    Err(e) => {
      println!("{}: {}", s, e);
      None
    }
  }
}

fn run(conf: Config) -> Option<bool> {
  let mut files = FxHashMap::default();
  let mut paths = FxHashMap::default();
  for path in conf.source {
    let contents = read_file(&path)?;
    let path = Path::new(&path).canonicalize().unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    files.insert(uri.clone(), contents);
    paths.insert(path, uri);
  }
  let db = Db::new(files);
  let diagnostics = db.all_diagnostics();
  for &(ref uri, ref ds) in diagnostics.iter() {
    for d in ds.iter() {
      println!("{}:{}", uri.path(), d);
    }
  }
  if conf.format {
    for (path, uri) in paths.iter() {
      let formatted = match db.format(uri) {
        None => {
          println!("cannot format {}: syntax error", path.display());
          return None;
        }
        Some(x) => x,
      };
      match std::fs::write(&path, formatted) {
        Ok(()) => {}
        Err(e) => {
          println!("{}: {}", path.display(), e);
          return None;
        }
      }
    }
  }
  if conf.cst {
    for (path, uri) in paths.iter() {
      println!("==> {}", path.display());
      print!("{:#?}", db.syntax(uri).unwrap());
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
      println!("no errors");
      0
    }
    Ok(Some(false)) => 1,
    Ok(None) => 2,
    Err(_) => 3,
  };
  std::process::exit(ec)
}
