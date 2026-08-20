use std::{net::SocketAddr, sync::Arc};

use bridgething_headless::{
  advertise,
  auth::Auth,
  config::{APP_NAME, Cli},
  hints::BroadcastHints,
  server::{self, Console},
  web,
};
use bridgething_host_shell::{Host, HostConfig, hints::HintSink, logs};
use clap::Parser;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let cli = Cli::parse();
  let verbosity = logs::install();

  let paths = cli.paths()?;
  let config_dir = paths.config_dir.clone();
  std::fs::create_dir_all(&config_dir)?;

  let auth = Auth::open(&config_dir, cli.token.clone(), cli.no_auth);
  let hints = Arc::new(BroadcastHints::new());
  // cloned as the concrete type, then unsized into the trait object
  let sink: Arc<dyn HintSink> = hints.clone();
  let host = Host::boot(HostConfig::new(APP_NAME, cli.gateway_url(), paths), sink, verbosity).await?;

  // an artifact only matters for the push that follows it, and an sd card is small
  let _ = tokio::fs::remove_dir_all(host.spool_dir()).await;

  let listener = TcpListener::bind(cli.bind).await?;
  let bound = listener.local_addr().unwrap_or(cli.bind);
  let _advertisement = (!cli.no_mdns).then(|| advertise::spawn(bound)).flatten();

  announce(bound, auth.token());
  let router = server::router(Console::new(host, hints, auth), web::resolve(cli.web_root));
  server::serve(listener, router).await?;
  Ok(())
}

fn announce(bound: SocketAddr, token: Option<&str>) {
  let port = bound.port();
  match token {
    Some(token) => tracing::info!("the console is up: http://<this-machine>:{port}/?token={token}"),
    None => tracing::warn!(
      "the console is up with no token: http://<this-machine>:{port}/ - anyone who can reach the port can drive it"
    ),
  }
}
