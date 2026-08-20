import type * as api from '@bridgething/companion-types';
import type { ConsoleSession, InstallOutcome, KnownDevice, OtaOutcome } from '@bridgething/console';
import type { Endpoint, Invalidation, Topic, WebappResource } from '@bridgething/ui';

import { call, eventsUrl } from './api.ts';

const HINTS: Record<string, Topic> = {
  'invalidate:session': 'session',
  'invalidate:endpoints': 'endpoints',
  'invalidate:providers': 'providers',
  'invalidate:peers': 'peers',
  'invalidate:now-playing': 'now-playing',
  'invalidate:ancs': 'ancs',
  'invalidate:device-meta': 'device-meta',
  'invalidate:webapps': 'webapps',
  'invalidate:webapp-doc': 'webapp-doc',
  'invalidate:voice-model': 'voice-model',
  'invalidate:ota-runs': 'ota-runs',
  'invalidate:ota-available': 'ota-available',
  'invalidate:ota-poll': 'ota-poll',
  'invalidate:logs': 'logs',
  'invalidate:known-devices': 'known-devices',
};

const RESYNC = 'invalidate:all';

const FIRST_RETRY = 500;
const RETRY_CAP = 10_000;

/**
 * The console over http. Every op is the one the desktop shell invokes over
 * tauri, and the hints arrive on a socket the server holds open instead of on
 * a webview event bus. A dropped socket is normal - a laptop sleeps, a pi
 * restarts - so it comes back on its own and asks for a full resync.
 */
export class HttpSession implements ConsoleSession {
  readonly tier = 'companion' as const;
  readonly host = 'console' as const;

  private readonly listeners = new Set<(event: Invalidation) => void>();
  private socket: WebSocket | null = null;
  private backoff = FIRST_RETRY;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private stopped = false;

  static async start(): Promise<HttpSession> {
    const session = new HttpSession();
    await session.watch();
    return session;
  }

  /** resolves once the first socket has settled, so the first paint is not empty */
  private watch(): Promise<void> {
    return new Promise(settle => {
      this.open(settle);
    });
  }

  private open(settle?: () => void): void {
    if (this.stopped) return;

    const socket = new WebSocket(eventsUrl());
    this.socket = socket;

    socket.addEventListener('open', () => {
      this.backoff = FIRST_RETRY;
      settle?.();
      settle = undefined;
    });

    socket.addEventListener('message', message => {
      const heard = read(message.data);
      if (!heard) return;
      if (heard.name === RESYNC) {
        for (const topic of Object.values(HINTS)) this.fan({ topic, id: null });
        return;
      }
      const topic = HINTS[heard.name];
      if (topic) this.fan({ topic, id: heard.id });
    });

    socket.addEventListener('close', () => {
      this.socket = null;
      settle?.();
      settle = undefined;
      this.retry();
    });

    socket.addEventListener('error', () => socket.close());
  }

  private retry(): void {
    if (this.stopped || this.timer) return;
    const wait = this.backoff;
    this.backoff = Math.min(this.backoff * 2, RETRY_CAP);
    this.timer = setTimeout(() => {
      this.timer = null;
      this.open();
    }, wait);
  }

  private fan(event: Invalidation): void {
    for (const listener of this.listeners) listener(event);
  }

  subscribe(listener: (event: Invalidation) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  stop(): void {
    this.stopped = true;
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
    this.socket?.close();
    this.socket = null;
    this.listeners.clear();
  }

  endpoints = () => call<Endpoint[]>('endpoints');
  defaultGateway = () => call<string>('default_gateway');
  connect = (url?: string) => call<string>('connect', { url: url ?? null });
  disconnect = () => call<void>('disconnect');

  knownDevices = () => call<KnownDevice[]>('known_devices');
  setDeviceAutoConnect = (url: string, enabled: boolean) => call<void>('set_device_auto_connect', { url, enabled });
  forgetKnownDevice = (url: string) => call<void>('forget_known_device', { url });

  selectedDevice = () => call<string | null>('selected_device');
  selectDevice = (deviceId: string | null) => call<void>('select_device', { deviceId });

  route = () => call<string>('route');
  setRoute = (path: string) => call<void>('set_route', { path });
  catalogSources = () => call<string[]>('catalog_sources');
  addCatalogSource = (url: string) => call<string[]>('add_catalog_source', { url });
  removeCatalogSource = (url: string) => call<string[]>('remove_catalog_source', { url });

  snapshot = () => call<api.SessionSnapshot>('session_snapshot');
  hostInfo = () => call<api.SessionHostInfo>('host_info');
  peers = () => call<api.SessionPeer[]>('peers');
  deviceMeta = () => call<api.DeviceMetaEntry[]>('device_meta');
  capabilities = () => call<api.CapabilityFlags>('capabilities');
  capabilitySupport = () => call<api.CapabilityFlags>('capability_support');
  setCapabilityFlags = (flags: api.CapabilityFlags) => call<void>('set_capability_flags', { flags });
  setDeviceNickname = (nickname: string) => call<void>('set_device_nickname', { nickname });
  deviceAutoResume = () => call<boolean>('device_auto_resume');
  setDeviceAutoResume = (enabled: boolean) => call<void>('set_device_auto_resume', { enabled });

  webapps = () => call<api.WebappInfo[]>('webapps');
  webappActive = () => call<api.ActiveWebapp | null>('webapp_active');
  webappSlots = () => call<api.WebappSlots>('webapp_slots');
  setWebappSlot = (slot: api.WebappSlot, id: string | null) => call<api.WebappSlots>('set_webapp_slot', { slot, id });
  switchWebapp = (id: string) => call<void>('switch_webapp', { id });
  uninstallWebapp = (id: string) => call<void>('uninstall_webapp', { id });
  installWebappFromUrl = (url: string, provenance?: string, expected?: api.ArtifactDigest | null) =>
    call<api.WebappInfo>('install_webapp_from_url', {
      url,
      expected: expected ?? null,
      provenance: provenance ?? null,
    });
  webappResource = (id: string, kind: api.WebappResourceKind) =>
    call<WebappResource>('webapp_resource', { id, kind });

  webappConfig = (id: string) => call<api.ConfigEntry[]>('webapp_config', { id });
  setWebappConfigField = (id: string, key: string, value: string) =>
    call<void>('set_webapp_config_field', { id, key, value });
  deleteWebappConfigField = (id: string, key: string) => call<void>('delete_webapp_config_field', { id, key });
  webappDoc = (id: string) => call<api.DocEntry[]>('webapp_doc', { id });
  webappDocEntry = (id: string, key: string) => call<string | null>('webapp_doc_entry', { id, key });
  setWebappDoc = (id: string, key: string, value: string) => call<void>('set_webapp_doc', { id, key, value });
  deleteWebappDoc = (id: string, key: string) => call<void>('delete_webapp_doc', { id, key });

  voiceModel = () => call<api.VoiceModelState>('voice_model');

  otaRuns = () => call<api.OtaRun[]>('ota_runs');
  otaAvailable = () => call<api.OtaAvailable[]>('ota_available');
  otaPoll = () => call<api.OtaPollStatus>('ota_poll');
  otaManifest = (rootUrl: string) => call<api.OtaDiscoverManifest>('ota_manifest', { rootUrl });
  setOtaPollConfig = (config: api.OtaPollConfig | null) => call<void>('set_ota_poll_config', { config });
  applyOtaUpdate = (channel: string, version: string, rootUrl: string) =>
    call<void>('apply_ota_update', { channel, version, rootUrl });
  checkForOtaUpdate = (rootUrl: string) => call<void>('ota_check_now', { rootUrl });
  dismissOtaRun = () => call<void>('ota_dismiss_run');

  otaPushDaemon = (artifact: string) => call<OtaOutcome>('ota_push_daemon', { artifact });
  otaInstallWebapp = (bundle: string, provenance?: string) =>
    call<InstallOutcome>('ota_install_webapp', { bundle, provenance: provenance ?? null });

  deviceLogs = (limit: number) => call<api.DeviceLogLine[]>('device_logs', { limit });
  deviceLogStreaming = () => call<boolean>('device_log_streaming');
  setDeviceLogStreaming = (enabled: boolean) => call<void>('set_device_log_streaming', { enabled });
  debugLogging = () => call<boolean>('debug_logging');
  setDebugLogging = (enabled: boolean) => call<void>('set_debug_logging', { enabled });

  nowPlaying = () => call<api.NowPlaying | null>('now_playing');
  providers = () => call<api.ProviderInfo[]>('providers');
  providerPriority = () => call<string[]>('provider_priority');
  libraryProvider = () => call<string | null>('library_provider');
  setProviderPriority = (ids: string[]) => call<void>('set_provider_priority', { ids });
  connectProvider = (id: string) => call<void>('connect_provider', { id });
  disconnectProvider = (id: string) => call<void>('disconnect_provider', { id });
  completeProviderAuth = (id: string, tokens: api.ProviderTokens) =>
    call<void>('complete_provider_auth', { id, tokens });
  cancelProviderAuth = (id: string) => call<void>('cancel_provider_auth', { id });
}

function read(payload: unknown): { name: string; id: string | null } | null {
  if (typeof payload !== 'string') return null;
  try {
    const held = JSON.parse(payload) as { name?: unknown; id?: unknown };
    if (typeof held.name !== 'string') return null;
    return { name: held.name, id: typeof held.id === 'string' ? held.id : null };
  } catch {
    return null;
  }
}
