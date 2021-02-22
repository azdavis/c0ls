//! This is the only file (other than main.rs) that may have side effects.

use crate::from::CrateFrom;
use crate::wrapper::{Handled, Notif, Req};
use analysis::{Db, Edit, Update};
use lsp_server::{Connection, Message, Response};
use lsp_types::notification::{
  DidChangeTextDocument, DidChangeWatchedFiles, PublishDiagnostics,
};
use lsp_types::request::{GotoDefinition, HoverRequest};
use lsp_types::{
  FileChangeType, GotoDefinitionResponse, InitializeParams,
  PublishDiagnosticsParams, Url,
};
use std::fs::read_to_string;
use walkdir::WalkDir;

pub(crate) fn run(conn: &Connection, init: InitializeParams) {
  log::info!("starting");
  let root = init.root_uri.expect("no root");
  let mut db = Db::new(get_initial_files(&root));
  send_all_diagnostics(conn, &db);
  for msg in conn.receiver.iter() {
    match msg {
      Message::Request(req) => {
        if conn.handle_shutdown(&req).unwrap() {
          log::info!("shutting down");
          return;
        }
        match handle_req(&db, Req::new(req)) {
          Ok(req) => log::warn!("ignoring request: {}", req.method()),
          Err(res) => conn.sender.send(res.into()).unwrap(),
        }
      }
      Message::Response(res) => log::warn!("ignoring response: {:?}", res),
      Message::Notification(notif) => {
        match handle_notif(&conn, &mut db, Notif::new(notif)) {
          Ok(notif) => log::warn!("ignoring notification: {}", notif.method()),
          Err(Handled) => {}
        }
      }
    }
  }
}

fn handle_req(db: &Db, req: Req) -> Result<Req, Response> {
  req
    .handle::<GotoDefinition, _>(|_, params| {
      log::info!("goto def");
      let params = params.text_document_position_params;
      let loc = db.go_to_def(
        &params.text_document.uri,
        CrateFrom::from(params.position),
      )?;
      Some(GotoDefinitionResponse::Scalar(CrateFrom::from(loc)))
    })?
    .handle::<HoverRequest, _>(|_, params| {
      log::info!("hover");
      let params = params.text_document_position_params;
      db.hover(&params.text_document.uri, CrateFrom::from(params.position))
        .map(CrateFrom::from)
    })
}

fn handle_notif(
  conn: &Connection,
  db: &mut Db,
  notif: Notif,
) -> Result<Notif, Handled> {
  notif
    .handle::<DidChangeWatchedFiles, _>(|params| {
      log::info!("watched files changed");
      db.update_files(params.changes.into_iter().map(
        |change| match change.typ {
          FileChangeType::Created | FileChangeType::Changed => {
            let contents = read_to_string(change.uri.path()).unwrap();
            Update::Create(change.uri, contents)
          }
          FileChangeType::Deleted => Update::Delete(change.uri),
        },
      ));
      send_all_diagnostics(conn, db);
    })?
    .handle::<DidChangeTextDocument, _>(|params| {
      log::info!("did change a text document");
      db.edit_file(
        &params.text_document.uri,
        params.content_changes.into_iter().map(|edit| Edit {
          range: edit.range.map(CrateFrom::from),
          text: edit.text,
        }),
      );
      send_all_diagnostics(conn, db);
    })
}

fn get_initial_files(root: &Url) -> impl Iterator<Item = (Url, String)> {
  WalkDir::new(root.path()).into_iter().filter_map(|entry| {
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
    let uri = Url::from_file_path(path).unwrap();
    let contents = read_to_string(entry.path()).unwrap();
    Some((uri, contents))
  })
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
