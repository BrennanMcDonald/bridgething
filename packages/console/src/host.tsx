import type { ComponentType, VNode } from 'preact';

export type ArtifactKind = 'daemon' | 'webapp';

/** Toggling whether the host comes back with the machine, where it can. */
export type Autostart = {
  get(): Promise<boolean>;
  set(enabled: boolean): Promise<void>;
};

/** Stopping the running host, where the user is allowed to. */
export type Lifecycle = {
  quit(): Promise<void>;
};

/**
 * The parts of a host the screens cannot reach through the session: picking a
 * file, handing one back, the clipboard, and the few controls that only make
 * sense on one host. A capability the host does not have is null, and the
 * screen that would use it leaves itself out.
 */
export interface HostAdapter {
  readonly kind: 'desktop' | 'web';

  /** the handle a picked artifact is known by, or null if the user backed out */
  pickArtifact(kind: ArtifactKind): Promise<string | null>;
  /** offers the body to the user as a file; false if they declined it */
  saveText(name: string, body: string): Promise<boolean>;
  copyText(text: string): Promise<void>;

  autostart: Autostart | null;
  lifecycle: Lifecycle | null;
  /** the host's own updater, when it has one to show */
  SelfUpdate: ComponentType<{ version: string | undefined }> | null;
}

let attached: HostAdapter | null = null;

export function attachHost(adapter: HostAdapter): void {
  attached = adapter;
}

export function host(): HostAdapter {
  if (!attached) throw new Error('a host adapter has to be attached before the console mounts');
  return attached;
}

export function saveText(name: string, body: string): Promise<boolean> {
  return host().saveText(name, body);
}

export function pickArtifact(kind: ArtifactKind): Promise<string | null> {
  return host().pickArtifact(kind);
}

export function copyText(text: string): Promise<void> {
  return host().copyText(text);
}

/** how a screen names the machine whose filesystem a typed path refers to */
export function hostOrigin(): string {
  return host().kind === 'web' ? 'the machine running bridgething' : 'this computer';
}

export function selfUpdate(version: string | undefined): VNode | null {
  const Update = host().SelfUpdate;
  return Update ? <Update version={version} /> : null;
}
