use lsp_types::{
  HoverOptions, HoverProviderCapability, ServerCapabilities,
  TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
  WorkDoneProgressOptions,TextDocumentSyncSaveOptions,
};

pub(crate) fn get() -> ServerCapabilities {
  ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Options(
      TextDocumentSyncOptions {
        open_close: Some(false),
        change: Some(TextDocumentSyncKind::Full),
        will_save: Some(false),
        will_save_wait_until: Some(false),
        save: Some(TextDocumentSyncSaveOptions::Supported(false)),
      },
    )),
    hover_provider: Some(HoverProviderCapability::Options(HoverOptions {
      work_done_progress_options: WorkDoneProgressOptions {
        work_done_progress: Some(false),
      },
    })),
    ..ServerCapabilities::default()
  }
}
