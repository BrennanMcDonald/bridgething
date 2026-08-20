use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
  Router,
  body::Body,
  extract::{
    ConnectInfo, DefaultBodyLimit, Path as AxumPath, Query, Request, State,
    ws::{Message, WebSocket, WebSocketUpgrade},
  },
  http::{StatusCode, header},
  middleware::{self, Next},
  response::{IntoResponse, Response},
  routing::{get, post},
};
use bridgething_host_shell::{Host, hints::RESYNC, ops::CommandError};
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{io::AsyncWriteExt as _, net::TcpListener};
use tower_http::services::{ServeDir, ServeFile};

use crate::{auth::Auth, hints::BroadcastHints, rpc, web};

/// A daemon image is the biggest thing anyone hands the console.
const MAX_ARTIFACT: u64 = 512 * 1024 * 1024;

const NO_CONSOLE: &str = "the console assets are not installed. build them with `bun run build` in headless/, \
or point --web-root at a directory holding index.html.";

#[derive(Clone)]
pub struct Console {
  host: Arc<Host>,
  hints: Arc<BroadcastHints>,
  auth: Auth,
}

impl Console {
  pub fn new(host: Arc<Host>, hints: Arc<BroadcastHints>, auth: Auth) -> Self {
    Self { host, hints, auth }
  }
}

pub fn router(console: Console, web_root: Option<PathBuf>) -> Router {
  let api = Router::new()
    .route("/rpc/{op}", post(rpc_handler))
    .route("/events", get(events_handler))
    .route("/artifact", post(artifact_handler).layer(DefaultBodyLimit::disable()))
    .route_layer(middleware::from_fn_with_state(console.clone(), guard))
    .with_state(console);

  match web_root {
    Some(root) => {
      let spa = ServeDir::new(&root).fallback(ServeFile::new(web::index_of(&root)));
      Router::new().nest("/api", api).fallback_service(spa)
    }
    None => {
      tracing::warn!("{NO_CONSOLE}");
      Router::new()
        .nest("/api", api)
        .fallback(|| async { (StatusCode::SERVICE_UNAVAILABLE, NO_CONSOLE) })
    }
  }
}

pub async fn serve(listener: TcpListener, router: Router) -> std::io::Result<()> {
  axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await
}

// MARK: auth

async fn guard(
  State(console): State<Console>,
  ConnectInfo(peer): ConnectInfo<SocketAddr>,
  request: Request,
  next: Next,
) -> Response {
  let bearer = request
    .headers()
    .get(header::AUTHORIZATION)
    .and_then(|held| held.to_str().ok())
    .and_then(|held| held.strip_prefix("Bearer "))
    .map(str::to_owned);

  // a browser cannot put a header on a websocket, so that one carries its token in the query
  let ticketed = request.uri().query().and_then(ticket).map(str::to_owned);

  let presented = bearer.or(ticketed);
  if console.auth.allows(presented.as_deref()) {
    return next.run(request).await;
  }

  tracing::warn!(%peer, path = %request.uri().path(), "a console request arrived without the token");
  (StatusCode::UNAUTHORIZED, "the console needs its token").into_response()
}

/// The token is url-safe by construction, so the query needs no decoding.
fn ticket(query: &str) -> Option<&str> {
  query.split('&').find_map(|pair| pair.strip_prefix("token="))
}

// MARK: ops

fn refused(error: CommandError) -> Response {
  let status = match &error {
    CommandError::NotConnected => StatusCode::CONFLICT,
    CommandError::Host(_) => StatusCode::BAD_REQUEST,
    _ => StatusCode::BAD_GATEWAY,
  };
  let body = serde_json::to_value(&error)
    .unwrap_or_else(|_| serde_json::json!({ "kind": "host", "reason": error.to_string() }));
  (status, axum::Json(body)).into_response()
}

async fn rpc_handler(State(console): State<Console>, AxumPath(op): AxumPath<String>, body: String) -> Response {
  let params = match body.trim() {
    "" => Value::Null,
    held => match serde_json::from_str(held) {
      Ok(params) => params,
      Err(error) => return refused(CommandError::Host(format!("{op}: {error}"))),
    },
  };
  match rpc::dispatch(&console.host, &op, params).await {
    Ok(value) => axum::Json(value).into_response(),
    Err(error) => {
      tracing::debug!(%op, %error, "an op did not answer");
      refused(error)
    }
  }
}

// MARK: events

async fn events_handler(State(console): State<Console>, upgrade: WebSocketUpgrade) -> Response {
  upgrade.on_upgrade(move |socket| fan(socket, console))
}

async fn fan(mut socket: WebSocket, console: Console) {
  let mut notices = console.hints.subscribe();

  // a console that has just opened knows nothing; tell it to pull everything
  if socket
    .send(Message::Text(format!(r#"{{"name":"{RESYNC}","id":null}}"#).into()))
    .await
    .is_err()
  {
    return;
  }

  loop {
    tokio::select! {
      heard = notices.recv() => match heard {
        Ok(notice) => {
          let Ok(json) = serde_json::to_string(&notice) else { continue };
          if socket.send(Message::Text(json.into())).await.is_err() {
            break;
          }
        }
        Err(tokio::sync::broadcast::error::RecvError::Lagged(missed)) => {
          tracing::debug!(missed, "a console fell behind the hints; it is told to resync");
          if socket
            .send(Message::Text(format!(r#"{{"name":"{RESYNC}","id":null}}"#).into()))
            .await
            .is_err()
          {
            break;
          }
        }
        Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
      },
      // the only inbound frames that matter are the close and the pings axum answers for us
      frame = socket.recv() => match frame {
        Some(Ok(_)) => {}
        _ => break,
      },
    }
  }

  tracing::debug!("a console let go of its event socket");
}

// MARK: artifacts

#[derive(Debug, Deserialize)]
struct Naming {
  name: Option<String>,
}

#[derive(Debug, Serialize)]
struct Spooled {
  path: String,
}

/// The browser has no path to hand the ops layer, so it hands over the bytes
/// and gets back the path they landed at.
async fn artifact_handler(State(console): State<Console>, Query(naming): Query<Naming>, body: Body) -> Response {
  let spool = console.host.spool_dir();
  if let Err(error) = tokio::fs::create_dir_all(spool).await {
    return refused(CommandError::Artifact(format!("{}: {error}", spool.display())));
  }

  let path = spool.join(format!(
    "{}-{}",
    uuid::Uuid::now_v7().simple(),
    sanitized(naming.name.as_deref())
  ));

  match write_body(&path, body).await {
    Ok(written) => {
      tracing::info!(path = %path.display(), written, "the console took an artifact");
      axum::Json(Spooled {
        path: path.to_string_lossy().into_owned(),
      })
      .into_response()
    }
    Err(reason) => {
      let _ = tokio::fs::remove_file(&path).await;
      refused(CommandError::Artifact(reason))
    }
  }
}

async fn write_body(path: &std::path::Path, body: Body) -> Result<u64, String> {
  let mut file = tokio::fs::File::create(path)
    .await
    .map_err(|error| format!("{}: {error}", path.display()))?;

  let mut written = 0u64;
  let mut chunks = std::pin::pin!(body.into_data_stream());
  while let Some(chunk) = chunks.next().await {
    let chunk = chunk.map_err(|error| format!("the upload stopped early: {error}"))?;
    written += chunk.len() as u64;
    if written > MAX_ARTIFACT {
      return Err(format!("an artifact over {MAX_ARTIFACT} bytes is not taken"));
    }
    file
      .write_all(&chunk)
      .await
      .map_err(|error| format!("{}: {error}", path.display()))?;
  }
  file.flush().await.map_err(|error| error.to_string())?;
  Ok(written)
}

fn sanitized(name: Option<&str>) -> String {
  let held = name.unwrap_or("artifact");
  let kept: String = held
    .chars()
    .filter(|held| held.is_ascii_alphanumeric() || matches!(held, '.' | '-' | '_'))
    .take(64)
    .collect();
  if kept.is_empty() || kept.starts_with('.') {
    "artifact".to_owned()
  } else {
    kept
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn an_uploaded_name_cannot_walk_out_of_the_spool() {
    assert_eq!(
      sanitized(Some("../../etc/passwd")),
      "artifact",
      "a name that reads as a traversal is not a name"
    );
    assert_eq!(sanitized(Some("my webapp.zip")), "mywebapp.zip");
    assert_eq!(sanitized(Some("bridgething-0.10.0.swu")), "bridgething-0.10.0.swu");
    assert_eq!(sanitized(Some("...")), "artifact");
    assert_eq!(sanitized(None), "artifact");
  }
}
