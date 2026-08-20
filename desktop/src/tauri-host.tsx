import type { ArtifactKind, HostAdapter } from '@bridgething/console';
import { invoke } from '@tauri-apps/api/core';
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { open, save } from '@tauri-apps/plugin-dialog';

import { SelfUpdate } from './SelfUpdate.tsx';

const FILTERS: Record<ArtifactKind, { name: string; extensions: string[] }> = {
  daemon: { name: 'daemon or system image', extensions: ['swu', 'zst', 'bin', '*'] },
  webapp: { name: 'webapp bundle', extensions: ['zip'] },
};

export const tauriHost: HostAdapter = {
  kind: 'desktop',

  async pickArtifact(kind) {
    const picked = await open({
      multiple: false,
      directory: false,
      title: kind === 'daemon' ? 'pick a daemon or image artifact' : 'pick a webapp bundle',
      filters: [FILTERS[kind]],
    });
    return typeof picked === 'string' ? picked : null;
  },

  async saveText(name, body) {
    const path = await save({
      title: 'save the log lines',
      defaultPath: name,
      filters: [{ name: 'log', extensions: ['log', 'txt'] }],
    }).catch(() => null);
    if (!path) return false;
    await invoke<void>('export_logs', { path, body });
    return true;
  },

  copyText: writeText,

  autostart: {
    get: isEnabled,
    set: async enabled => {
      if (enabled) await enable();
      else await disable();
    },
  },

  lifecycle: { quit: () => invoke<void>('quit') },

  SelfUpdate,
};
