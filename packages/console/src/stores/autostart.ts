import { host } from '../host.tsx';

import { resource } from './resource.ts';

export const autostart = resource(false, async () => (await host().autostart?.get()) ?? false);

export async function setAutostart(enabled: boolean): Promise<void> {
  await host().autostart?.set(enabled);
  await autostart.refresh();
}
