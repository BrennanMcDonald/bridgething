import type * as api from '@bridgething/companion-types';
import { isCompanion, useSession, type CompanionSession } from '@bridgething/ui';

export type KnownDevice = { url: string; name: string; autoConnect: boolean; lastConnectedAt: string | null };

export type OtaOutcome = { kind: 'completed' } | { kind: 'failed'; reason: string } | { kind: 'interrupted' };

export type InstallOutcome = { kind: 'installed'; id: string } | { kind: 'failed'; reason: string };

/**
 * What every host surface answers, whatever it is mounted over. The two
 * implementations are the desktop shell's tauri bridge and the headless
 * server's http one; both are backed by the same rust ops.
 */
export interface ConsoleSession extends CompanionSession {
  readonly host: 'console';

  capabilitySupport(): Promise<api.CapabilityFlags>;
  defaultGateway(): Promise<string>;

  knownDevices(): Promise<KnownDevice[]>;
  setDeviceAutoConnect(url: string, enabled: boolean): Promise<void>;
  forgetKnownDevice(url: string): Promise<void>;

  selectedDevice(): Promise<string | null>;
  selectDevice(deviceId: string | null): Promise<void>;

  route(): Promise<string>;
  setRoute(path: string): Promise<void>;

  catalogSources(): Promise<string[]>;
  addCatalogSource(url: string): Promise<string[]>;
  removeCatalogSource(url: string): Promise<string[]>;

  deviceAutoResume(): Promise<boolean>;
  deviceLogStreaming(): Promise<boolean>;

  debugLogging(): Promise<boolean>;
  setDebugLogging(enabled: boolean): Promise<void>;

  /** both take an artifact handle minted by the host adapter's picker */
  otaPushDaemon(artifact: string): Promise<OtaOutcome>;
  otaInstallWebapp(bundle: string, provenance?: string): Promise<InstallOutcome>;
}

export function isConsole(session: CompanionSession): session is ConsoleSession {
  return (session as Partial<ConsoleSession>).host === 'console';
}

export function useConsole(): ConsoleSession {
  const session = useSession();
  if (!isCompanion(session)) throw new Error(`the console needs a companion backend, not a ${session.tier} one`);
  if (!isConsole(session)) throw new Error('the console is mounted over a backend that is not a host one');
  return session;
}
