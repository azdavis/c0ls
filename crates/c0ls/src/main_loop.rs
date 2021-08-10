//! This is the only file (other than main.rs) that may have side effects.

use crate::from::CrateFrom;
use crate::wrapper::{Handled, Notif, Req};
use analysis::{Db, Edit, Update};
use lsp_server::{Connection, Message, Response};
use lsp_types::notification::{
  DidChangeTextDocument, DidChangeWatchedFiles, PublishDiagnostics, ShowMessage,
};
use lsp_types::request::{GotoDefinition, HoverRequest};
use lsp_types::{
  FileChangeType, GotoDefinitionResponse, InitializeParams, MessageType,
  PublishDiagnosticsParams, ShowMessageParams, Url,
};
use std::fs::read_to_string;
use walkdir::WalkDir;

pub(crate) fn run(conn: &Connection, init: InitializeParams) {
  log::info!("starting");
  let root = match init.root_uri {
    None => {
      show_error(conn, "cannot activate c0ls: no root".to_owned());
      return;
    }
    Some(x) => x,
  };
  let mut db = Db::new(get_initial_files(conn, &root));
  send_all_diagnostics(conn, &db);
  for msg in conn.receiver.iter() {
    match msg {
      Message::Request(req) => {
        if conn
          .handle_shutdown(&req)
          .expect("couldn't handle shutdown")
        {
          log::info!("shutting down");
          return;
        }
        match handle_req(&db, Req::new(req)) {
          Ok(req) => log::warn!("ignoring request: {}", req.method()),
          Err(res) => conn
            .sender
            .send(res.into())
            .expect("couldn't send response"),
        }
      }
      Message::Response(res) => log::warn!("ignoring response: {:?}", res),
      Message::Notification(notif) => {
        match handle_notif(conn, &mut db, Notif::new(notif)) {
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
      db.update_files(params.changes.into_iter().filter_map(|change| {
        match change.typ {
          FileChangeType::Created | FileChangeType::Changed => {
            let path = change.uri.path();
            match read_to_string(path) {
              Ok(contents) => Some(Update::Create(change.uri, contents)),
              Err(e) => {
                show_error(conn, format!("{}: {}", path, e));
                None
              }
            }
          }
          FileChangeType::Deleted => Some(Update::Delete(change.uri)),
        }
      }));
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

fn get_initial_files<'c>(
  conn: &'c Connection,
  root: &Url,
) -> impl Iterator<Item = (Url, String)> + 'c {
  WalkDir::new(root.path())
    .into_iter()
    .filter_map(move |entry| {
      let entry = match entry {
        Ok(x) => x,
        Err(e) => {
          show_error(conn, e.to_string());
          return None;
        }
      };
      let path = entry.path();
      if !path.is_file() {
        return None;
      }
      let ext = path.extension()?;
      if ext != "c0" && ext != "h0" {
        return None;
      }
      let uri = Url::from_file_path(path).expect("bad path");
      match read_to_string(entry.path()) {
        Ok(contents) => Some((uri, contents)),
        Err(e) => {
          show_error(conn, format!("{}: {}", path.display(), e));
          None
        }
      }
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
      .expect("couldn't send diagnostics");
  }
}

fn mk_notif<N>(val: N::Params) -> Message
where
  N: lsp_types::notification::Notification,
{
  Message::Notification(lsp_server::Notification {
    method: N::METHOD.to_owned(),
    params: serde_json::to_value(val).expect("couldn't make JSON"),
  })
}

fn show_error(conn: &Connection, message: String) {
  conn
    .sender
    .send(mk_notif::<ShowMessage>(ShowMessageParams {
      typ: MessageType::Error,
      message,
    }))
    .expect("couldn't show error")
}
