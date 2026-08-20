import { App, attachHost, seed } from '@bridgething/console';
import { SessionProvider } from '@bridgething/ui';
import { render } from 'preact';

import './app.css';
import { tauriHost } from './tauri-host.tsx';
import { TauriSession } from './tauri-session.ts';

const root = document.getElementById('app');
if (!root) throw new Error('the shell template is missing its mount point');

attachHost(tauriHost);
const session = await TauriSession.start();
await seed(session);

render(
  <SessionProvider session={session}>
    <App />
  </SessionProvider>,
  root,
);
