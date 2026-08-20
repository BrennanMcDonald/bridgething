use std::{path::PathBuf, sync::Arc};

use bridgething_companion::api::{
  ActiveWebapp, CapabilityFlags, ConfigEntry, DeviceLogLine, DeviceMetaEntry, DocEntry, NowPlaying, OtaPollConfig,
  ProviderInfo, ProviderTokens, SessionHostInfo, SessionPeer, SessionSnapshot, VoiceModelState, WebappInfo, WebappSlot,
  WebappSlots,
  ota::{ArtifactDigest, OtaAvailable, OtaDiscoverManifest, OtaPollStatus, OtaRun},
};
use bridgething_delivery::discovery::Endpoint;
use bridgething_host_shell::{
  Host,
  known_device::KnownDevice,
  ops::{self, Answer, InstallOutcome, OtaOutcome, WebappResource},
};
use libbridgething::gateway::WebappResourceKind;
use tauri::{AppHandle, Runtime, State};

#[tauri::command]
pub async fn session_snapshot(host: State<'_, Arc<Host>>) -> Answer<SessionSnapshot> {
  ops::session_snapshot(&host).await
}

#[tauri::command]
pub async fn host_info(host: State<'_, Arc<Host>>) -> Answer<SessionHostInfo> {
  ops::host_info(&host).await
}

#[tauri::command]
pub async fn capabilities(host: State<'_, Arc<Host>>) -> Answer<CapabilityFlags> {
  ops::capabilities(&host).await
}

#[tauri::command]
pub async fn capability_support(host: State<'_, Arc<Host>>) -> Answer<CapabilityFlags> {
  ops::capability_support(&host).await
}

#[tauri::command]
pub async fn providers(host: State<'_, Arc<Host>>) -> Answer<Vec<ProviderInfo>> {
  ops::providers(&host).await
}

#[tauri::command]
pub async fn provider_priority(host: State<'_, Arc<Host>>) -> Answer<Vec<String>> {
  ops::provider_priority(&host).await
}

#[tauri::command]
pub async fn library_provider(host: State<'_, Arc<Host>>) -> Answer<Option<String>> {
  ops::library_provider(&host).await
}

#[tauri::command]
pub async fn peers(host: State<'_, Arc<Host>>) -> Answer<Vec<SessionPeer>> {
  ops::peers(&host).await
}

#[tauri::command]
pub async fn now_playing(host: State<'_, Arc<Host>>) -> Answer<Option<NowPlaying>> {
  ops::now_playing(&host).await
}

#[tauri::command]
pub async fn device_meta(host: State<'_, Arc<Host>>) -> Answer<Vec<DeviceMetaEntry>> {
  ops::device_meta(&host).await
}

#[tauri::command]
pub async fn device_auto_resume(host: State<'_, Arc<Host>>) -> Answer<bool> {
  ops::device_auto_resume(&host).await
}

#[tauri::command]
pub async fn device_log_streaming(host: State<'_, Arc<Host>>) -> Answer<bool> {
  ops::device_log_streaming(&host).await
}

#[tauri::command]
pub async fn debug_logging(host: State<'_, Arc<Host>>) -> Answer<bool> {
  ops::debug_logging(&host).await
}

#[tauri::command]
pub async fn voice_model(host: State<'_, Arc<Host>>) -> Answer<VoiceModelState> {
  ops::voice_model(&host).await
}

#[tauri::command]
pub async fn ota_runs(host: State<'_, Arc<Host>>) -> Answer<Vec<OtaRun>> {
  ops::ota_runs(&host).await
}

#[tauri::command]
pub async fn ota_available(host: State<'_, Arc<Host>>) -> Answer<Vec<OtaAvailable>> {
  ops::ota_available(&host).await
}

#[tauri::command]
pub async fn ota_poll(host: State<'_, Arc<Host>>) -> Answer<OtaPollStatus> {
  ops::ota_poll(&host).await
}

#[tauri::command]
pub async fn webapps(host: State<'_, Arc<Host>>) -> Answer<Vec<WebappInfo>> {
  ops::webapps(&host).await
}

#[tauri::command]
pub async fn webapp_active(host: State<'_, Arc<Host>>) -> Answer<Option<ActiveWebapp>> {
  ops::webapp_active(&host).await
}

#[tauri::command]
pub async fn webapp_slots(host: State<'_, Arc<Host>>) -> Answer<WebappSlots> {
  ops::webapp_slots(&host).await
}

#[tauri::command]
pub async fn webapp_config(host: State<'_, Arc<Host>>, id: String) -> Answer<Vec<ConfigEntry>> {
  ops::webapp_config(&host, id).await
}

#[tauri::command]
pub async fn webapp_doc(host: State<'_, Arc<Host>>, id: String) -> Answer<Vec<DocEntry>> {
  ops::webapp_doc(&host, id).await
}

#[tauri::command]
pub async fn webapp_doc_entry(host: State<'_, Arc<Host>>, id: String, key: String) -> Answer<Option<String>> {
  ops::webapp_doc_entry(&host, id, key).await
}

#[tauri::command]
pub async fn device_logs(host: State<'_, Arc<Host>>, limit: u32) -> Answer<Vec<DeviceLogLine>> {
  ops::device_logs(&host, limit).await
}

#[tauri::command]
pub async fn export_logs(path: PathBuf, body: String) -> Answer<()> {
  ops::export_logs(path, body).await
}

#[tauri::command]
pub async fn ota_manifest(host: State<'_, Arc<Host>>, root_url: String) -> Answer<OtaDiscoverManifest> {
  ops::ota_manifest(&host, root_url).await
}

#[tauri::command]
pub async fn webapp_resource(
  host: State<'_, Arc<Host>>,
  id: String,
  kind: WebappResourceKind,
) -> Answer<WebappResource> {
  ops::webapp_resource(&host, id, kind).await
}

#[tauri::command]
pub async fn endpoints(host: State<'_, Arc<Host>>) -> Answer<Vec<Endpoint>> {
  ops::endpoints(&host).await
}

#[tauri::command]
pub async fn default_gateway(host: State<'_, Arc<Host>>) -> Answer<String> {
  ops::default_gateway(&host).await
}

#[tauri::command]
pub async fn route(host: State<'_, Arc<Host>>) -> Answer<String> {
  ops::route(&host).await
}

#[tauri::command]
pub async fn set_route(host: State<'_, Arc<Host>>, path: String) -> Answer<()> {
  ops::set_route(&host, path).await
}

#[tauri::command]
pub async fn catalog_sources(host: State<'_, Arc<Host>>) -> Answer<Vec<String>> {
  ops::catalog_sources(&host).await
}

#[tauri::command]
pub async fn add_catalog_source(host: State<'_, Arc<Host>>, url: String) -> Answer<Vec<String>> {
  ops::add_catalog_source(&host, url).await
}

#[tauri::command]
pub async fn remove_catalog_source(host: State<'_, Arc<Host>>, url: String) -> Answer<Vec<String>> {
  ops::remove_catalog_source(&host, url).await
}

#[tauri::command]
pub async fn connect(host: State<'_, Arc<Host>>, url: Option<String>) -> Answer<String> {
  ops::connect(&host, url).await
}

#[tauri::command]
pub async fn disconnect(host: State<'_, Arc<Host>>, device_id: Option<String>) -> Answer<()> {
  ops::disconnect(&host, device_id).await
}

#[tauri::command]
pub async fn known_devices(host: State<'_, Arc<Host>>) -> Answer<Vec<KnownDevice>> {
  ops::known_devices(&host).await
}

#[tauri::command]
pub async fn set_device_auto_connect(host: State<'_, Arc<Host>>, url: String, enabled: bool) -> Answer<()> {
  ops::set_device_auto_connect(&host, url, enabled).await
}

#[tauri::command]
pub async fn forget_known_device(host: State<'_, Arc<Host>>, url: String) -> Answer<()> {
  ops::forget_known_device(&host, url).await
}

#[tauri::command]
pub async fn selected_device(host: State<'_, Arc<Host>>) -> Answer<Option<String>> {
  ops::selected_device(&host).await
}

#[tauri::command]
pub async fn select_device(host: State<'_, Arc<Host>>, device_id: Option<String>) -> Answer<()> {
  ops::select_device(&host, device_id).await
}

#[tauri::command]
pub async fn set_provider_priority(host: State<'_, Arc<Host>>, ids: Vec<String>) -> Answer<()> {
  ops::set_provider_priority(&host, ids).await
}

#[tauri::command]
pub async fn connect_provider(host: State<'_, Arc<Host>>, id: String) -> Answer<()> {
  ops::connect_provider(&host, id).await
}

#[tauri::command]
pub async fn disconnect_provider(host: State<'_, Arc<Host>>, id: String) -> Answer<()> {
  ops::disconnect_provider(&host, id).await
}

#[tauri::command]
pub async fn cancel_provider_auth(host: State<'_, Arc<Host>>, id: String) -> Answer<()> {
  ops::cancel_provider_auth(&host, id).await
}

#[tauri::command]
pub async fn complete_provider_auth(host: State<'_, Arc<Host>>, id: String, tokens: ProviderTokens) -> Answer<()> {
  ops::complete_provider_auth(&host, id, tokens).await
}

#[tauri::command]
pub async fn set_capability_flags(host: State<'_, Arc<Host>>, flags: CapabilityFlags) -> Answer<()> {
  ops::set_capability_flags(&host, flags).await
}

#[tauri::command]
pub async fn set_device_auto_resume(host: State<'_, Arc<Host>>, enabled: bool) -> Answer<()> {
  ops::set_device_auto_resume(&host, enabled).await
}

#[tauri::command]
pub async fn set_device_log_streaming(host: State<'_, Arc<Host>>, enabled: bool) -> Answer<()> {
  ops::set_device_log_streaming(&host, enabled).await
}

#[tauri::command]
pub async fn set_debug_logging(host: State<'_, Arc<Host>>, enabled: bool) -> Answer<()> {
  ops::set_debug_logging(&host, enabled).await
}

#[tauri::command]
pub async fn set_device_nickname(host: State<'_, Arc<Host>>, nickname: String) -> Answer<()> {
  ops::set_device_nickname(&host, nickname).await
}

#[tauri::command]
pub async fn switch_webapp(host: State<'_, Arc<Host>>, id: String) -> Answer<()> {
  ops::switch_webapp(&host, id).await
}

#[tauri::command]
pub async fn uninstall_webapp(host: State<'_, Arc<Host>>, id: String) -> Answer<()> {
  ops::uninstall_webapp(&host, id).await
}

#[tauri::command]
pub async fn set_webapp_slot(
  host: State<'_, Arc<Host>>,
  slot: WebappSlot,
  id: Option<String>,
) -> Answer<WebappSlots> {
  ops::set_webapp_slot(&host, slot, id).await
}

#[tauri::command]
pub async fn set_webapp_config_field(
  host: State<'_, Arc<Host>>,
  id: String,
  key: String,
  value: String,
) -> Answer<()> {
  ops::set_webapp_config_field(&host, id, key, value).await
}

#[tauri::command]
pub async fn delete_webapp_config_field(host: State<'_, Arc<Host>>, id: String, key: String) -> Answer<()> {
  ops::delete_webapp_config_field(&host, id, key).await
}

#[tauri::command]
pub async fn set_webapp_doc(host: State<'_, Arc<Host>>, id: String, key: String, value: String) -> Answer<()> {
  ops::set_webapp_doc(&host, id, key, value).await
}

#[tauri::command]
pub async fn delete_webapp_doc(host: State<'_, Arc<Host>>, id: String, key: String) -> Answer<()> {
  ops::delete_webapp_doc(&host, id, key).await
}

#[tauri::command]
pub async fn set_ota_poll_config(host: State<'_, Arc<Host>>, config: Option<OtaPollConfig>) -> Answer<()> {
  ops::set_ota_poll_config(&host, config).await
}

#[tauri::command]
pub async fn apply_ota_update(
  host: State<'_, Arc<Host>>,
  channel: String,
  version: String,
  root_url: String,
) -> Answer<()> {
  ops::apply_ota_update(&host, channel, version, root_url).await
}

#[tauri::command]
pub async fn ota_push_daemon(host: State<'_, Arc<Host>>, artifact: PathBuf) -> Answer<OtaOutcome> {
  ops::ota_push_daemon(&host, artifact).await
}

#[tauri::command]
pub async fn ota_install_webapp(
  host: State<'_, Arc<Host>>,
  bundle: PathBuf,
  provenance: Option<String>,
) -> Answer<InstallOutcome> {
  ops::ota_install_webapp(&host, bundle, provenance).await
}

#[tauri::command]
pub async fn install_webapp_from_url(
  host: State<'_, Arc<Host>>,
  url: String,
  expected: Option<ArtifactDigest>,
  provenance: Option<String>,
) -> Answer<WebappInfo> {
  ops::install_webapp_from_url(&host, url, expected, provenance).await
}

#[tauri::command]
pub async fn ota_check_now(host: State<'_, Arc<Host>>, root_url: String) -> Answer<()> {
  ops::ota_check_now(&host, root_url).await
}

#[tauri::command]
pub async fn ota_dismiss_run(host: State<'_, Arc<Host>>) -> Answer<()> {
  ops::ota_dismiss_run(&host).await
}

#[tauri::command]
pub fn restart<R: Runtime>(app: AppHandle<R>) {
  crate::process::restart(&app)
}

#[tauri::command]
pub fn quit<R: Runtime>(app: AppHandle<R>) {
  crate::process::leave(&app)
}
