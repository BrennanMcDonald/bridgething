use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use bridgething_host_shell::hints::{Hint, HintSink, Invalidation};
use tauri::{AppHandle, Emitter, Runtime};

#[derive(Clone)]
pub struct Visibility(Arc<AtomicBool>);

impl Visibility {
  pub fn shown() -> Self {
    Self(Arc::new(AtomicBool::new(true)))
  }

  pub fn hidden() -> Self {
    Self(Arc::new(AtomicBool::new(false)))
  }

  pub fn set(&self, visible: bool) {
    self.0.store(visible, Ordering::Relaxed);
  }

  pub fn get(&self) -> bool {
    self.0.load(Ordering::Relaxed)
  }
}

pub struct WindowHints<R: Runtime> {
  handle: AppHandle<R>,
  visible: Visibility,
}

impl<R: Runtime> WindowHints<R> {
  pub fn new(handle: AppHandle<R>, visible: Visibility) -> Self {
    Self { handle, visible }
  }
}

impl<R: Runtime> HintSink for WindowHints<R> {
  fn emit(&self, hint: Hint) {
    if !self.visible.get() {
      return;
    }
    if let Err(error) = self.handle.emit(hint.name, Invalidation { id: hint.id }) {
      tracing::warn!(%error, name = hint.name, "an invalidation hint could not be emitted");
    }
  }
}
