use bridgething_host_shell::hints::{Hint, HintSink};
use serde::Serialize;
use tokio::sync::broadcast;

const BACKLOG: usize = 256;

/// What one invalidation looks like on the console's event socket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Notice {
  pub name: &'static str,
  pub id: Option<String>,
}

/// The desktop shell hands its hints to a window. With no window to hand them
/// to, they fan out to every console the server is holding a socket for.
pub struct BroadcastHints(broadcast::Sender<Notice>);

impl BroadcastHints {
  pub fn new() -> Self {
    Self(broadcast::channel(BACKLOG).0)
  }

  pub fn subscribe(&self) -> broadcast::Receiver<Notice> {
    self.0.subscribe()
  }
}

impl Default for BroadcastHints {
  fn default() -> Self {
    Self::new()
  }
}

impl HintSink for BroadcastHints {
  fn emit(&self, hint: Hint) {
    let _ = self.0.send(Notice {
      name: hint.name,
      id: hint.id,
    });
  }
}
