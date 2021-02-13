//! This is the only file (other than main.rs) that may have side effects.

use crate::from::CrateFrom;
use crate::wrapper::{Handled, Notif, Req};
use analysis::Db;
use lsp_server::{Connection, Message, Response};
use lsp_types::notification::{DidChangeWatchedFiles, PublishDiagnostics};
use lsp_types::request::HoverRequest;
use lsp_types::{InitializeParams, PublishDiagnosticsParams, Url};
use rustc_hash::FxHashMap;
use std::fs::read_to_string;
use walkdir::WalkDir;

pub(crate) fn run(conn: &Connection, init: InitializeParams) {
  eprintln!("starting main loop");
  let root = init.root_uri.expect("no root");
  let mut db = Db::new(get_initial_files(&root));
  send_all_diagnostics(conn, &db);
  for msg in conn.receiver.iter() {
    match msg {
      Message::Request(req) => {
        if conn.handle_shutdown(&req).unwrap() {
          eprintln!("shutting down");
          return;
        }
        match handle_req(&db, Req::new(req)) {
          Ok(req) => eprintln!("don't know how to handle {}", req.method()),
          Err(res) => conn.sender.send(res.into()).unwrap(),
        }
      }
      Message::Response(res) => eprintln!("got response: {:?}", res),
      Message::Notification(notif) => {
        match handle_notif(&conn, &root, &mut db, Notif::new(notif)) {
          Ok(notif) => eprintln!("don't know how to handle {}", notif.method()),
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
  conn: &Connection,
  root: &Url,
  db: &mut Db,
  notif: Notif,
) -> Result<Notif, Handled> {
  notif.handle::<DidChangeWatchedFiles, _>(|_| {
    // TODO impl incremental updating
    *db = Db::new(get_initial_files(root));
    send_all_diagnostics(conn, db);
  })
}

fn get_initial_files(root: &Url) -> FxHashMap<Url, String> {
  WalkDir::new(root.path())
    .into_iter()
    .filter_map(|entry| {
      let entry = entry.unwrap();
      let path = entry.path();
      if !path.is_file() {
        return None;
      }
      let ext = path.extension()?;
      if ext != "c0" && ext != "h0" {
        return None;
      }
      let path = path.as_os_str().to_str().unwrap();
      eprintln!("got file: {}", path);
      let uri = Url::from_file_path(path).unwrap();
      let contents = read_to_string(entry.path()).unwrap();
      Some((uri, contents))
    })
    .collect()
}

fn send_all_diagnostics(conn: &Connection, db: &Db) {
  for (uri, diagnostics) in db.all_diagnostics() {
    let params = PublishDiagnosticsParams {
      uri,
      diagnostics: diagnostics.into_iter().map(CrateFrom::from).collect(),
      version: None,
    };
    conn
      .sender
      .send(mk_notif::<PublishDiagnostics>(params))
      .unwrap();
  }
}

fn mk_notif<N>(val: N::Params) -> Message
where
  N: lsp_types::notification::Notification,
{
  Message::Notification(lsp_server::Notification {
    method: N::METHOD.to_owned(),
    params: serde_json::to_value(val).unwrap(),
  })
}