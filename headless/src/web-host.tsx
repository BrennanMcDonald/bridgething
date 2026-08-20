import type { ArtifactKind, HostAdapter } from '@bridgething/console';

import { upload } from './api.ts';

const ACCEPT: Record<ArtifactKind, string> = {
  daemon: '.swu,.zst,.bin,application/octet-stream',
  webapp: '.zip,application/zip',
};

/** Opens the browser's own file picker and resolves once the user has chosen. */
function chooseFile(accept: string): Promise<File | null> {
  return new Promise(settle => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = accept;
    input.style.display = 'none';
    // firefox needs the input in the document for `cancel` to fire
    document.body.append(input);

    const done = (file: File | null) => {
      input.remove();
      settle(file);
    };

    input.addEventListener('change', () => done(input.files?.[0] ?? null), { once: true });
    input.addEventListener('cancel', () => done(null), { once: true });
    input.click();
  });
}

/**
 * The browser cannot reach the machine's filesystem, so a picked artifact is
 * uploaded and known afterwards by the path it landed at, and a saved file is
 * handed back through a download instead of a save dialog.
 */
export const webHost: HostAdapter = {
  kind: 'web',

  async pickArtifact(kind) {
    const file = await chooseFile(ACCEPT[kind]);
    return file ? await upload(file) : null;
  },

  async saveText(name, body) {
    const url = URL.createObjectURL(new Blob([body], { type: 'text/plain' }));
    const link = document.createElement('a');
    link.href = url;
    link.download = name;
    link.click();
    URL.revokeObjectURL(url);
    return true;
  },

  async copyText(text) {
    // a console reached over plain http is not a secure context, so the
    // clipboard api is not there and a hidden selection has to do it
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      return;
    }

    const holder = document.createElement('textarea');
    holder.value = text;
    holder.setAttribute('readonly', '');
    holder.style.position = 'fixed';
    holder.style.opacity = '0';
    document.body.append(holder);
    holder.select();
    const copied = document.execCommand('copy');
    holder.remove();
    if (!copied) throw new Error('this browser would not copy without a secure context');
  },

  // systemd starts the console with the machine, and stopping it from a page
  // it serves would leave nothing to start it again
  autostart: null,
  lifecycle: null,
  SelfUpdate: null,
};
