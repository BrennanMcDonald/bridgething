use std::{
  path::PathBuf,
  sync::{Arc, OnceLock},
};

use bridgething_delivery::discovery::{Discovery, Endpoint};

use crate::{
  autoconnect,
  hints::{ENDPOINTS, Hint, HintSink},
  logs::{self, Verbosity},
  route::Route,
  shell::{HostPaths, Shell, ShellConfig, ShellError},
  sources::Sources,
};

pub struct HostConfig {
  pub app_name: String,
  pub gateway_url: String,
  pub paths: HostPaths,
}

impl HostConfig {
  pub fn new(app_name: impl Into<String>, gateway_url: impl Into<String>, paths: HostPaths) -> Self {
    Self {
      app_name: app_name.into(),
      gateway_url: gateway_url.into(),
      paths,
    }
  }
}

/// Everything a host surface needs behind one handle: the linked shell, the
/// endpoints mdns found, and the small files that outlive a launch.
pub struct Host {
  shell: Arc<Shell>,
  discovery: OnceLock<Arc<Discovery>>,
  sources: Sources,
  route: Route,
  verbosity: Arc<Verbosity>,
  spool: PathBuf,
}

impl Host {
  /// Builds the host without the loops that reach out on their own. A surface
  /// that wants those calls [`Host::boot`]; a test that wants to drive the
  /// shell by hand calls this.
  pub fn assemble(
    config: HostConfig,
    hints: Arc<dyn HintSink>,
    verbosity: Arc<Verbosity>,
  ) -> Result<Arc<Self>, ShellError> {
    let config_dir = config.paths.config_dir.clone();
    let spool = config.paths.cache_dir.join("spool");
    let shell = Shell::create(
      ShellConfig::new(config.app_name, config.gateway_url, config.paths),
      hints,
    )?;

    logs::attach(shell.session().log_inbox());

    Ok(Arc::new(Self {
      shell,
      discovery: OnceLock::new(),
      sources: Sources::open(&config_dir),
      route: Route::open(&config_dir),
      verbosity,
      spool,
    }))
  }

  pub async fn boot(
    config: HostConfig,
    hints: Arc<dyn HintSink>,
    verbosity: Arc<Verbosity>,
  ) -> Result<Arc<Self>, ShellError> {
    let host = Self::assemble(config, Arc::clone(&hints), verbosity)?;
    host.shell.start().await;

    let wake = host.shell.wake();
    match Discovery::spawn(move |_| {
      hints.emit(Hint::bare(ENDPOINTS));
      wake.notify_one();
    }) {
      Ok(discovery) => {
        let _ = host.discovery.set(discovery);
      }
      Err(error) => tracing::warn!(%error, "mdns is not answering; devices have to be dialed by address"),
    }

    autoconnect::spawn(Arc::clone(&host.shell), {
      let host = Arc::downgrade(&host);
      move || host.upgrade().map(|host| host.endpoints()).unwrap_or_default()
    });

    Ok(host)
  }

  pub fn shell(&self) -> &Arc<Shell> {
    &self.shell
  }

  pub fn endpoints(&self) -> Vec<Endpoint> {
    self
      .discovery
      .get()
      .map(|found| found.endpoints())
      .unwrap_or_default()
  }

  pub fn sources(&self) -> &Sources {
    &self.sources
  }

  pub fn route(&self) -> &Route {
    &self.route
  }

  pub fn verbosity(&self) -> &Arc<Verbosity> {
    &self.verbosity
  }

  /// Where an artifact handed to the host lands before it is pushed at a device.
  pub fn spool_dir(&self) -> &PathBuf {
    &self.spool
  }
}
