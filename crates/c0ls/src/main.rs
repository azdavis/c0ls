//! A language server for C0.

mod capabilities;
mod from;
mod main_loop;
mod wrapper;

use lsp_server::Connection;
use lsp_types::InitializeParams;

fn main() {
  env_logger::init();
  let (conn, io_threads) = Connection::stdio();
  let sc = serde_json::to_value(&capabilities::get()).unwrap();
  let init = conn.initialize(sc).unwrap();
  let init: InitializeParams = serde_json::from_value(init).unwrap();
  main_loop::run(&conn, init);
  io_threads.join().unwrap();
}
