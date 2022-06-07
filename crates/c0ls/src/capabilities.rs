use lsp_types::{
  HoverProviderCapability, OneOf, ServerCapabilities,
  TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
  TextDocumentSyncSaveOptions,
};

pub(crate) fn get() -> ServerCapabilities {
  ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Options(
      TextDocumentSyncOptions {
        open_close: Some(false),
        change: Some(TextDocumentSyncKind::INCREMENTAL),
        will_save: Some(false),
        will_save_wait_until: Some(false),
        save: Some(TextDocumentSyncSaveOptions::Supported(false)),
      },
    )),
    definition_provider: Some(OneOf::Left(true)),
    hover_provider: Some(HoverProviderCapability::Simple(true)),
    ..ServerCapabilities::default()
  }
}
