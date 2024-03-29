use lsp_server::{Notification, Request, RequestId, Response};

#[derive(Debug)]
pub(crate) struct Req(Request);

impl Req {
  pub(crate) fn new(req: Request) -> Self {
    Self(req)
  }

  pub(crate) fn handle<R, F>(self, f: F) -> Result<Self, Response>
  where
    R: lsp_types::request::Request,
    F: FnOnce(RequestId, R::Params) -> R::Result,
  {
    match self.0.clone().extract::<R::Params>(R::METHOD) {
      Ok((id, params)) => {
        let result = f(id.clone(), params);
        let val = serde_json::to_value(&result).expect("couldn't make JSON");
        Err(Response {
          id,
          result: Some(val),
          error: None,
        })
      }
      Err(_) => Ok(self),
    }
  }

  pub(crate) fn method(&self) -> &str {
    self.0.method.as_str()
  }
}

#[derive(Debug)]
pub(crate) struct Notif(Notification);

#[derive(Debug, Clone, Copy)]
pub(crate) struct Handled;

impl Notif {
  pub(crate) fn new(notif: Notification) -> Self {
    Self(notif)
  }

  pub(crate) fn handle<N, F>(self, f: F) -> Result<Self, Handled>
  where
    N: lsp_types::notification::Notification,
    F: FnOnce(N::Params),
  {
    match self.0.clone().extract::<N::Params>(N::METHOD) {
      Ok(params) => {
        f(params);
        Err(Handled)
      }
      Err(_) => Ok(self),
    }
  }

  pub(crate) fn method(&self) -> &str {
    self.0.method.as_str()
  }
}
