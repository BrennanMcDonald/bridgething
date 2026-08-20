use std::net::SocketAddr;

use mdns_sd::{ServiceDaemon, ServiceInfo};

const SERVICE_TYPE: &str = "_http._tcp.local.";
const INSTANCE: &str = "bridgething console";

/// A headless box has no screen to read its own address off, so the console
/// answers mdns the way the daemon does. The handle keeps the registration
/// alive; dropping it withdraws the announcement.
pub struct Advertisement {
  _daemon: ServiceDaemon,
}

pub fn spawn(bind: SocketAddr) -> Option<Advertisement> {
  let daemon = ServiceDaemon::new()
    .inspect_err(|error| tracing::warn!(%error, "the console cannot answer mdns"))
    .ok()?;

  let host = hostname();
  let info = ServiceInfo::new(
    SERVICE_TYPE,
    INSTANCE,
    &format!("{host}.local."),
    "",
    bind.port(),
    &[("path", "/")][..],
  )
  .inspect_err(|error| tracing::warn!(%error, "the console announcement is not well formed"))
  .ok()?
  .enable_addr_auto();

  daemon
    .register(info)
    .inspect_err(|error| tracing::warn!(%error, "the console announcement did not go out"))
    .ok()?;

  tracing::info!(host = %format!("{host}.local"), port = bind.port(), "the console is answering mdns");
  Some(Advertisement { _daemon: daemon })
}

fn hostname() -> String {
  std::fs::read_to_string("/etc/hostname")
    .ok()
    .map(|held| held.trim().to_owned())
    .filter(|held| !held.is_empty())
    .unwrap_or_else(|| "bridgething-console".to_owned())
}
