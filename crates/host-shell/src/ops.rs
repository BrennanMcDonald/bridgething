use std::{path::PathBuf, sync::Arc};

use bridgething_companion::api::{
  ActiveWebapp, CapabilityFlags, CompanionError, ConfigEntry, DeviceLogLine, DeviceMetaEntry, DocEntry, NowPlaying,
  OtaPollConfig, ProviderInfo, ProviderTokens, SessionHostInfo, SessionPeer, SessionSnapshot, VoiceModelState,
  WebappInfo, WebappSlot, WebappSlots,
  ota::{ArtifactDigest, OtaAvailable, OtaDiscoverManifest, OtaPollStatus, OtaRun},
};
use bridgething_delivery::{
  discovery::Endpoint,
  ota::{event::OtaPhaseSnapshot, service::WebappInstallResult, stream::FileSource},
  seam::BlobStore,
  transfer::FragmentSource,
};
use libbridgething::gateway::WebappResourceKind;
use serde::Serialize;
use uuid::Uuid;

use crate::{
  hints::{self, Hint},
  host::Host,
  known_device::KnownDevice,
  shell::{Shell, ShellError},
};

#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", content = "reason", rename_all = "camelCase")]
pub enum CommandError {
  #[error("no link to a daemon")]
  NotConnected,
  #[error("{0}")]
  Link(String),
  #[error("{0}")]
  Device(String),
  #[error("{0}")]
  Artifact(String),
  #[error("{0}")]
  Host(String),
}

impl From<ShellError> for CommandError {
  fn from(error: ShellError) -> Self {
    match error {
      ShellError::NotConnected => Self::NotConnected,
      other => Self::Link(other.to_string()),
    }
  }
}

impl From<CompanionError> for CommandError {
  fn from(error: CompanionError) -> Self {
    match error {
      CompanionError::NotConnected => Self::NotConnected,
      CompanionError::Cancelled => Self::Device("cancelled".to_owned()),
      CompanionError::ResourceNotAvailable => Self::Artifact("resource not available".to_owned()),
      CompanionError::Runtime(reason) => Self::Link(reason),
      CompanionError::Device(reason) => Self::Device(reason),
    }
  }
}

pub type Answer<T> = Result<T, CommandError>;

fn webapp_id(raw: &str) -> Answer<Uuid> {
  Uuid::parse_str(raw).map_err(|_| CommandError::Device(format!("not a webapp id: {raw}")))
}

fn peer(shell: &Shell) -> Answer<String> {
  shell.peer().ok_or(CommandError::NotConnected)
}

// MARK: pulls

pub async fn session_snapshot(host: &Host) -> Answer<SessionSnapshot> {
  Ok(host.shell().session().snapshot().await)
}

pub async fn host_info(host: &Host) -> Answer<SessionHostInfo> {
  Ok(host.shell().session().snapshot().await.host_info)
}

pub async fn capabilities(host: &Host) -> Answer<CapabilityFlags> {
  Ok(host.shell().session().snapshot().await.capability_flags)
}

pub async fn capability_support(host: &Host) -> Answer<CapabilityFlags> {
  Ok(host.shell().capability_support())
}

pub async fn providers(host: &Host) -> Answer<Vec<ProviderInfo>> {
  Ok(host.shell().session().snapshot().await.providers)
}

pub async fn provider_priority(host: &Host) -> Answer<Vec<String>> {
  Ok(host.shell().session().snapshot().await.provider_priority)
}

pub async fn library_provider(host: &Host) -> Answer<Option<String>> {
  Ok(host.shell().session().snapshot().await.library_provider)
}

pub async fn peers(host: &Host) -> Answer<Vec<SessionPeer>> {
  Ok(host.shell().session().snapshot().await.peers)
}

pub async fn now_playing(host: &Host) -> Answer<Option<NowPlaying>> {
  Ok(host.shell().session().snapshot().await.now_playing)
}

pub async fn device_meta(host: &Host) -> Answer<Vec<DeviceMetaEntry>> {
  Ok(host.shell().session().snapshot().await.device_meta)
}

pub async fn device_auto_resume(host: &Host) -> Answer<bool> {
  let Some(device_id) = host.shell().peer() else {
    return Ok(true);
  };
  Ok(
    host
      .shell()
      .session()
      .companion_debug()
      .auto_resume
      .into_iter()
      .find(|pref| pref.device_id == device_id)
      .map(|pref| pref.enabled)
      .unwrap_or(true),
  )
}

pub async fn device_log_streaming(host: &Host) -> Answer<bool> {
  Ok(host.shell().log_streaming())
}

pub async fn debug_logging(host: &Host) -> Answer<bool> {
  Ok(host.verbosity().get())
}

pub async fn voice_model(host: &Host) -> Answer<VoiceModelState> {
  Ok(host.shell().session().snapshot().await.voice_model)
}

pub async fn ota_runs(host: &Host) -> Answer<Vec<OtaRun>> {
  Ok(host.shell().session().snapshot().await.ota_runs)
}

pub async fn ota_available(host: &Host) -> Answer<Vec<OtaAvailable>> {
  Ok(host.shell().session().snapshot().await.ota_available)
}

pub async fn ota_poll(host: &Host) -> Answer<OtaPollStatus> {
  Ok(host.shell().session().snapshot().await.ota_poll)
}

pub async fn webapps(host: &Host) -> Answer<Vec<WebappInfo>> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().list_webapps(device_id).await?)
}

pub async fn webapp_active(host: &Host) -> Answer<Option<ActiveWebapp>> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().current_webapp(device_id).await?)
}

pub async fn webapp_slots(host: &Host) -> Answer<WebappSlots> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().webapp_slots(device_id).await?)
}

pub async fn webapp_config(host: &Host, id: String) -> Answer<Vec<ConfigEntry>> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().list_webapp_config(device_id, id).await?)
}

pub async fn webapp_doc(host: &Host, id: String) -> Answer<Vec<DocEntry>> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().list_webapp_doc(device_id, id).await?)
}

pub async fn webapp_doc_entry(host: &Host, id: String, key: String) -> Answer<Option<String>> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().get_webapp_doc(device_id, id, key).await?)
}

pub async fn device_logs(host: &Host, limit: u32) -> Answer<Vec<DeviceLogLine>> {
  Ok(host.shell().session().device_log_snapshot(limit))
}

pub async fn export_logs(path: PathBuf, body: String) -> Answer<()> {
  std::fs::write(&path, body).map_err(|reason| CommandError::Host(format!("{}: {reason}", path.display())))
}

pub async fn ota_manifest(host: &Host, root_url: String) -> Answer<OtaDiscoverManifest> {
  Ok(host.shell().session().fetch_ota_manifest(root_url).await?)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebappResource {
  pub digest: String,
  pub mime: Option<String>,
  pub bytes: Vec<u8>,
}

pub async fn webapp_resource(host: &Host, id: String, kind: WebappResourceKind) -> Answer<WebappResource> {
  let id = webapp_id(&id)?;
  let link = host.shell().link()?;
  let cached = host
    .shell()
    .resources()?
    .fetch(&link, id, kind)
    .await
    .map_err(|error| CommandError::Device(format!("{error:?}")))?;
  let bytes = host
    .shell()
    .blobs()
    .get(&cached.digest)
    .map_err(CommandError::Artifact)?
    .ok_or_else(|| CommandError::Artifact(format!("the resource store lost {}", cached.digest)))?;
  Ok(WebappResource {
    digest: cached.digest,
    mime: cached.mime,
    bytes,
  })
}

// MARK: actions

pub async fn endpoints(host: &Host) -> Answer<Vec<Endpoint>> {
  Ok(host.endpoints())
}

pub async fn default_gateway(host: &Host) -> Answer<String> {
  Ok(host.shell().gateway_url().to_owned())
}

pub async fn route(host: &Host) -> Answer<String> {
  Ok(host.route().get())
}

pub async fn set_route(host: &Host, path: String) -> Answer<()> {
  host.route().set(path);
  Ok(())
}

pub async fn catalog_sources(host: &Host) -> Answer<Vec<String>> {
  Ok(host.sources().list())
}

pub async fn add_catalog_source(host: &Host, url: String) -> Answer<Vec<String>> {
  Ok(host.sources().add(url))
}

pub async fn remove_catalog_source(host: &Host, url: String) -> Answer<Vec<String>> {
  Ok(host.sources().remove(&url))
}

pub async fn connect(host: &Host, url: Option<String>) -> Answer<String> {
  Ok(host.shell().connect(url).await?)
}

pub async fn disconnect(host: &Host, device_id: Option<String>) -> Answer<()> {
  host.shell().disconnect(device_id).await;
  Ok(())
}

pub async fn known_devices(host: &Host) -> Answer<Vec<KnownDevice>> {
  Ok(host.shell().known_devices())
}

pub async fn set_device_auto_connect(host: &Host, url: String, enabled: bool) -> Answer<()> {
  host.shell().set_auto_connect(&url, enabled);
  Ok(())
}

pub async fn forget_known_device(host: &Host, url: String) -> Answer<()> {
  host.shell().forget_device(&url);
  Ok(())
}

pub async fn selected_device(host: &Host) -> Answer<Option<String>> {
  Ok(host.shell().peer())
}

pub async fn select_device(host: &Host, device_id: Option<String>) -> Answer<()> {
  host.shell().select(device_id);
  Ok(())
}

pub async fn set_provider_priority(host: &Host, ids: Vec<String>) -> Answer<()> {
  host.shell().session().set_provider_priority(ids).await;
  Ok(())
}

pub async fn connect_provider(host: &Host, id: String) -> Answer<()> {
  Ok(host.shell().session().connect_provider(id).await?)
}

pub async fn disconnect_provider(host: &Host, id: String) -> Answer<()> {
  host.shell().session().disconnect_provider(id).await;
  Ok(())
}

pub async fn cancel_provider_auth(host: &Host, id: String) -> Answer<()> {
  host.shell().session().cancel_auth(id).await;
  Ok(())
}

pub async fn complete_provider_auth(host: &Host, id: String, tokens: ProviderTokens) -> Answer<()> {
  Ok(host.shell().session().complete_provider_auth(id, tokens).await?)
}

pub async fn set_capability_flags(host: &Host, flags: CapabilityFlags) -> Answer<()> {
  host.shell().set_capability_flags(flags).await;
  host.shell().announce(Hint::bare(hints::SESSION));
  Ok(())
}

pub async fn set_device_auto_resume(host: &Host, enabled: bool) -> Answer<()> {
  let device_id = peer(host.shell())?;
  host
    .shell()
    .session()
    .set_device_auto_resume(device_id.clone(), enabled)
    .await;
  host.shell().announce(Hint::about(hints::DEVICE_META, device_id));
  Ok(())
}

pub async fn set_device_log_streaming(host: &Host, enabled: bool) -> Answer<()> {
  host.shell().set_log_streaming(enabled).await;
  host.shell().announce(Hint::bare(hints::LOGS));
  Ok(())
}

pub async fn set_debug_logging(host: &Host, enabled: bool) -> Answer<()> {
  host.verbosity().set(enabled);
  tracing::info!(enabled, "the host log verbosity changed");
  Ok(())
}

pub async fn set_device_nickname(host: &Host, nickname: String) -> Answer<()> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().device_set_nickname(device_id, nickname).await?)
}

pub async fn switch_webapp(host: &Host, id: String) -> Answer<()> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().switch_webapp(device_id, id).await?)
}

pub async fn uninstall_webapp(host: &Host, id: String) -> Answer<()> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().uninstall_webapp(device_id, id).await?)
}

pub async fn set_webapp_slot(host: &Host, slot: WebappSlot, id: Option<String>) -> Answer<WebappSlots> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().set_webapp_slot(device_id, slot, id).await?)
}

pub async fn set_webapp_config_field(host: &Host, id: String, key: String, value: String) -> Answer<()> {
  let device_id = peer(host.shell())?;
  Ok(
    host
      .shell()
      .session()
      .set_webapp_config_field(device_id, id, key, value)
      .await?,
  )
}

pub async fn delete_webapp_config_field(host: &Host, id: String, key: String) -> Answer<()> {
  let device_id = peer(host.shell())?;
  Ok(
    host
      .shell()
      .session()
      .delete_webapp_config_field(device_id, id, key)
      .await?,
  )
}

pub async fn set_webapp_doc(host: &Host, id: String, key: String, value: String) -> Answer<()> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().set_webapp_doc(device_id, id, key, value).await?)
}

pub async fn delete_webapp_doc(host: &Host, id: String, key: String) -> Answer<()> {
  let device_id = peer(host.shell())?;
  Ok(host.shell().session().delete_webapp_doc(device_id, id, key).await?)
}

pub async fn set_ota_poll_config(host: &Host, config: Option<OtaPollConfig>) -> Answer<()> {
  host.shell().session().set_ota_poll_config(config).await;
  host.shell().announce(Hint::bare(hints::OTA_POLL));
  Ok(())
}

pub async fn apply_ota_update(host: &Host, channel: String, version: String, root_url: String) -> Answer<()> {
  let device_id = peer(host.shell())?;
  host
    .shell()
    .session()
    .apply_ota_update(device_id, channel, version, root_url)
    .await;
  Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum OtaOutcome {
  Completed,
  Failed { reason: String },
  Interrupted,
}

impl From<OtaPhaseSnapshot> for OtaOutcome {
  fn from(phase: OtaPhaseSnapshot) -> Self {
    match phase {
      OtaPhaseSnapshot::Completed => Self::Completed,
      OtaPhaseSnapshot::Failed { reason } => Self::Failed { reason },
      _ => Self::Interrupted,
    }
  }
}

pub async fn ota_push_daemon(host: &Host, artifact: PathBuf) -> Answer<OtaOutcome> {
  let device_id = peer(host.shell())?;
  Ok(
    host
      .shell()
      .ota()
      .push_daemon(&device_id, spool(artifact)?, None)
      .await
      .into(),
  )
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum InstallOutcome {
  Installed { id: String },
  Failed { reason: String },
}

pub async fn ota_install_webapp(host: &Host, bundle: PathBuf, provenance: Option<String>) -> Answer<InstallOutcome> {
  let device_id = peer(host.shell())?;
  let bundle = spool(bundle)?;
  let outcome = host
    .shell()
    .ota()
    .install_webapp(&device_id, bundle, provenance.as_deref())
    .await;
  Ok(match outcome {
    WebappInstallResult::Installed(info) => InstallOutcome::Installed {
      id: info.id.to_string(),
    },
    WebappInstallResult::Failed { reason } => InstallOutcome::Failed { reason },
  })
}

pub async fn install_webapp_from_url(
  host: &Host,
  url: String,
  expected: Option<ArtifactDigest>,
  provenance: Option<String>,
) -> Answer<WebappInfo> {
  let device_id = peer(host.shell())?;
  Ok(
    host
      .shell()
      .session()
      .install_webapp_from_url(device_id, url, expected, provenance)
      .await?,
  )
}

pub async fn ota_check_now(host: &Host, root_url: String) -> Answer<()> {
  host.shell().ota().check_now(&root_url).await;
  Ok(())
}

pub async fn ota_dismiss_run(host: &Host) -> Answer<()> {
  let device_id = peer(host.shell())?;
  host.shell().ota().dismiss_run(&device_id).await;
  Ok(())
}

fn spool(path: PathBuf) -> Answer<Arc<FileSource>> {
  let source = Arc::new(FileSource::open(&path));
  let mut probe = [0u8; 1];
  source
    .read_at(0, &mut probe)
    .map_err(|reason| CommandError::Artifact(format!("{}: {reason}", path.display())))?;
  Ok(source)
}
