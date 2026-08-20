use bridgething_host_shell::{
  Host,
  ops::{self, Answer, CommandError},
};
use serde::de::DeserializeOwned;
use serde_json::Value;

fn field<T: DeserializeOwned>(params: &Value, name: &str) -> Answer<T> {
  serde_json::from_value(params.get(name).cloned().unwrap_or(Value::Null))
    .map_err(|error| CommandError::Host(format!("{name}: {error}")))
}

fn json<T: serde::Serialize>(value: T) -> Answer<Value> {
  serde_json::to_value(value).map_err(|error| CommandError::Host(error.to_string()))
}

/// The console speaks the same op names the desktop shell invokes over tauri,
/// against the same implementations. A name that is not here is not an op.
pub async fn dispatch(host: &Host, op: &str, params: Value) -> Answer<Value> {
  let at = &params;
  match op {
    // MARK: pulls
    "session_snapshot" => json(ops::session_snapshot(host).await?),
    "host_info" => json(ops::host_info(host).await?),
    "capabilities" => json(ops::capabilities(host).await?),
    "capability_support" => json(ops::capability_support(host).await?),
    "providers" => json(ops::providers(host).await?),
    "provider_priority" => json(ops::provider_priority(host).await?),
    "library_provider" => json(ops::library_provider(host).await?),
    "peers" => json(ops::peers(host).await?),
    "now_playing" => json(ops::now_playing(host).await?),
    "device_meta" => json(ops::device_meta(host).await?),
    "device_auto_resume" => json(ops::device_auto_resume(host).await?),
    "device_log_streaming" => json(ops::device_log_streaming(host).await?),
    "debug_logging" => json(ops::debug_logging(host).await?),
    "voice_model" => json(ops::voice_model(host).await?),
    "ota_runs" => json(ops::ota_runs(host).await?),
    "ota_available" => json(ops::ota_available(host).await?),
    "ota_poll" => json(ops::ota_poll(host).await?),
    "webapps" => json(ops::webapps(host).await?),
    "webapp_active" => json(ops::webapp_active(host).await?),
    "webapp_slots" => json(ops::webapp_slots(host).await?),
    "webapp_config" => json(ops::webapp_config(host, field(at, "id")?).await?),
    "webapp_doc" => json(ops::webapp_doc(host, field(at, "id")?).await?),
    "webapp_doc_entry" => json(ops::webapp_doc_entry(host, field(at, "id")?, field(at, "key")?).await?),
    "webapp_resource" => json(ops::webapp_resource(host, field(at, "id")?, field(at, "kind")?).await?),
    "device_logs" => json(ops::device_logs(host, field(at, "limit")?).await?),
    "ota_manifest" => json(ops::ota_manifest(host, field(at, "rootUrl")?).await?),
    "endpoints" => json(ops::endpoints(host).await?),
    "default_gateway" => json(ops::default_gateway(host).await?),
    "route" => json(ops::route(host).await?),
    "catalog_sources" => json(ops::catalog_sources(host).await?),
    "known_devices" => json(ops::known_devices(host).await?),
    "selected_device" => json(ops::selected_device(host).await?),

    // MARK: actions
    "set_route" => json(ops::set_route(host, field(at, "path")?).await?),
    "add_catalog_source" => json(ops::add_catalog_source(host, field(at, "url")?).await?),
    "remove_catalog_source" => json(ops::remove_catalog_source(host, field(at, "url")?).await?),
    "connect" => json(ops::connect(host, field(at, "url")?).await?),
    "disconnect" => json(ops::disconnect(host, field(at, "deviceId")?).await?),
    "set_device_auto_connect" => {
      json(ops::set_device_auto_connect(host, field(at, "url")?, field(at, "enabled")?).await?)
    }
    "forget_known_device" => json(ops::forget_known_device(host, field(at, "url")?).await?),
    "select_device" => json(ops::select_device(host, field(at, "deviceId")?).await?),
    "set_provider_priority" => json(ops::set_provider_priority(host, field(at, "ids")?).await?),
    "connect_provider" => json(ops::connect_provider(host, field(at, "id")?).await?),
    "disconnect_provider" => json(ops::disconnect_provider(host, field(at, "id")?).await?),
    "cancel_provider_auth" => json(ops::cancel_provider_auth(host, field(at, "id")?).await?),
    "complete_provider_auth" => json(ops::complete_provider_auth(host, field(at, "id")?, field(at, "tokens")?).await?),
    "set_capability_flags" => json(ops::set_capability_flags(host, field(at, "flags")?).await?),
    "set_device_auto_resume" => json(ops::set_device_auto_resume(host, field(at, "enabled")?).await?),
    "set_device_log_streaming" => json(ops::set_device_log_streaming(host, field(at, "enabled")?).await?),
    "set_debug_logging" => json(ops::set_debug_logging(host, field(at, "enabled")?).await?),
    "set_device_nickname" => json(ops::set_device_nickname(host, field(at, "nickname")?).await?),
    "switch_webapp" => json(ops::switch_webapp(host, field(at, "id")?).await?),
    "uninstall_webapp" => json(ops::uninstall_webapp(host, field(at, "id")?).await?),
    "set_webapp_slot" => json(ops::set_webapp_slot(host, field(at, "slot")?, field(at, "id")?).await?),
    "set_webapp_config_field" => {
      json(ops::set_webapp_config_field(host, field(at, "id")?, field(at, "key")?, field(at, "value")?).await?)
    }
    "delete_webapp_config_field" => {
      json(ops::delete_webapp_config_field(host, field(at, "id")?, field(at, "key")?).await?)
    }
    "set_webapp_doc" => {
      json(ops::set_webapp_doc(host, field(at, "id")?, field(at, "key")?, field(at, "value")?).await?)
    }
    "delete_webapp_doc" => json(ops::delete_webapp_doc(host, field(at, "id")?, field(at, "key")?).await?),
    "set_ota_poll_config" => json(ops::set_ota_poll_config(host, field(at, "config")?).await?),
    "apply_ota_update" => {
      json(ops::apply_ota_update(host, field(at, "channel")?, field(at, "version")?, field(at, "rootUrl")?).await?)
    }
    "ota_push_daemon" => json(ops::ota_push_daemon(host, field(at, "artifact")?).await?),
    "ota_install_webapp" => json(ops::ota_install_webapp(host, field(at, "bundle")?, field(at, "provenance")?).await?),
    "install_webapp_from_url" => {
      let (url, expected) = (field(at, "url")?, field(at, "expected")?);
      json(ops::install_webapp_from_url(host, url, expected, field(at, "provenance")?).await?)
    }
    "ota_check_now" => json(ops::ota_check_now(host, field(at, "rootUrl")?).await?),
    "ota_dismiss_run" => json(ops::ota_dismiss_run(host).await?),

    unknown => Err(CommandError::Host(format!("no such op: {unknown}"))),
  }
}
