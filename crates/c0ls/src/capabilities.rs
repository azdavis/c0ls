use lsp_types::{
  HoverOptions, HoverProviderCapability, ServerCapabilities,
  WorkDoneProgressOptions,
};

pub(crate) fn get() -> ServerCapabilities {
  ServerCapabilities {
    hover_provider: Some(HoverProviderCapability::Options(HoverOptions {
      work_done_progress_options: WorkDoneProgressOptions {
        work_done_progress: Some(false),
      },
    })),
    ..ServerCapabilities::default()
  }
}
