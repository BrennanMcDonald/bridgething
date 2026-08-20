use std::{net::SocketAddr, path::PathBuf};

use bridgething_host_shell::{HostPaths, ShellError, shell::gateway_url_from_env};
use clap::Parser;

pub const APP_NAME: &str = "bridgething console";

/// The bridgething companion host for a machine with no desktop: it holds the
/// link to a Car Thing and serves the console over http.
#[derive(Debug, Parser)]
#[command(name = "bridgething-headless", version, about, long_about = None)]
pub struct Cli {
  /// where the console listens
  #[arg(long, default_value = "0.0.0.0:8899", env = "BRIDGETHING_CONSOLE_BIND")]
  pub bind: SocketAddr,

  /// the daemon gateway to dial when the console asks for a link by hand
  #[arg(long, env = "BRIDGETHING_GATEWAY_URL")]
  pub gateway_url: Option<String>,

  /// state, cache, and config under one root instead of the xdg directories
  #[arg(long, env = "BRIDGETHING_CONSOLE_DATA_DIR")]
  pub data_dir: Option<PathBuf>,

  /// the built console assets; defaults to `web/` beside the binary
  #[arg(long, env = "BRIDGETHING_CONSOLE_WEB")]
  pub web_root: Option<PathBuf>,

  /// the token the console has to present; one is minted and kept if this is not given
  #[arg(long, env = "BRIDGETHING_CONSOLE_TOKEN")]
  pub token: Option<String>,

  /// serve the console to anyone who can reach the port
  #[arg(long)]
  pub no_auth: bool,

  /// do not answer mdns queries for this console
  #[arg(long)]
  pub no_mdns: bool,
}

impl Cli {
  pub fn paths(&self) -> Result<HostPaths, ShellError> {
    match &self.data_dir {
      Some(root) => Ok(HostPaths::under(root)),
      None => HostPaths::xdg(),
    }
  }

  pub fn gateway_url(&self) -> String {
    self.gateway_url.clone().unwrap_or_else(gateway_url_from_env)
  }
}
