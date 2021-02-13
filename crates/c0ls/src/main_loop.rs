//! This is the only file (other than main.rs) that may have side effects.

use crate::from::CrateFrom;
use crate::wrapper::{Handled, Notif, Req};
use analysis::Db;
use lsp_server::{Connection, Message, Response};
use lsp_types::notification::DidChangeWatchedFiles;
use lsp_types::request::HoverRequest;
use lsp_types::{InitializeParams, Url};
use rustc_hash::FxHashMap;
use std::fs::read_to_string;
use walkdir::WalkDir;

pub(crate) fn run(conn: &Connection, init: InitializeParams) {
  eprintln!("starting main loop");
  let root = init.root_uri.expect("no root");
  let mut db = Db::new(get_initial_files(&root));
  for msg in conn.receiver.iter() {
    match msg {
      Message::Request(req) => {
        if conn.handle_shutdown(&req).unwrap() {
          eprintln!("shutting down");
          return;
        }
        eprintln!("got request: {:?}", req);
        match handle_req(&db, Req::new(req)) {
          Ok(_) => eprintln!("don't know how to handle"),
          Err(res) => conn.sender.send(res.into()).unwrap(),
        }
      }
      Message::Response(res) => {
        eprintln!("got response, ignoring: {:?}", res);
      }
      Message::Notification(notif) => {
        eprintln!("got notification: {:?}", notif);
        match handle_notif(&root, &mut db, Notif::new(notif)) {
          Ok(_) => eprintln!("don't know how to handle"),
          Err(Handled) => {}
        }
      }
    }
  }
}

fn handle_req(db: &Db, req: Req) -> Result<Req, Response> {
  req.handle::<HoverRequest, _>(|_, params| {
    let params = params.text_document_position_params;
    db.hover(&params.text_document.uri, CrateFrom::from(params.position))
      .map(CrateFrom::from)
  })
}

fn handle_notif(
  root: &Url,
  db: &mut Db,
  notif: Notif,
) -> Result<Notif, Handled> {
  notif.handle::<DidChangeWatchedFiles, _>(|_| {
    // TODO impl incremental updating
    *db = Db::new(get_initial_files(root));
  })
}

fn get_initial_files(root: &Url) -> FxHashMap<Url, String> {
  WalkDir::new(root.path())
    .into_iter()
    .map(|entry| {
      let entry = entry.unwrap();
      let path = entry.path().as_os_str().to_str().unwrap();
      let uri = Url::from_file_path(path).unwrap();
      let contents = read_to_string(entry.path()).unwrap();
      (uri, contents)
    })
    .collect()
}
