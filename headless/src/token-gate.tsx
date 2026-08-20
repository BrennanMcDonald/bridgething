import { Button, Field } from '@bridgething/ui';
import type { VNode } from 'preact';
import { useState } from 'preact/hooks';

import { keepToken } from './api.ts';

/**
 * The url the server prints carries the token. Someone who opened the bare
 * address instead lands here, where the line out of the log will do.
 */
export function TokenGate({ onToken }: { onToken: () => void }): VNode {
  const [held, setHeld] = useState('');

  const submit = () => {
    const token = held.trim();
    if (!token) return;
    keepToken(token);
    onToken();
  };

  return (
    <div class="flex h-full items-center justify-center bg-screen p-6">
      <div class="flex w-full max-w-md flex-col gap-4 border border-rule bg-surface p-6">
        <div class="flex flex-col gap-1">
          <h1 class="m-0 font-display text-screen-title text-off-white">this console needs its token</h1>
          <p class="m-0 text-hint leading-relaxed text-muted">
            it is in the url the server printed when it started, and on the machine itself at
            <span class="select-all font-mono"> /var/lib/bridgething-console/config/console-token</span>.
          </p>
        </div>
        <Field value={held} onInput={setHeld} onCommit={submit} placeholder="paste the token" type="password" />
        <Button variant="primary" onClick={submit}>
          open the console
        </Button>
      </div>
    </div>
  );
}
