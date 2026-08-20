import { App, attachHost, seed } from '@bridgething/console';
import { SessionProvider } from '@bridgething/ui';
import { render } from 'preact';

import './app.css';
import { call } from './api.ts';
import { HttpSession } from './http-session.ts';
import { TokenGate } from './token-gate.tsx';
import { webHost } from './web-host.tsx';

const root = document.getElementById('app');
if (!root) throw new Error('the console template is missing its mount point');

attachHost(webHost);

if (await admitted()) {
  const session = await HttpSession.start();
  await seed(session);
  render(
    <SessionProvider session={session}>
      <App />
    </SessionProvider>,
    root,
  );
} else {
  render(<TokenGate onToken={() => location.reload()} />, root);
}

/**
 * One cheap op decides whether this console is carrying the token the server
 * wants. Anything other than a refusal is the console's problem to show.
 */
async function admitted(): Promise<boolean> {
  return await call('host_info').then(
    () => true,
    (reason: unknown) => (reason as { kind?: string }).kind !== 'unauthorized',
  );
}
