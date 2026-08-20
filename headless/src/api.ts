import { describeError } from '@bridgething/ui';

const TOKEN_KEY = 'bridgething.console-token';

/**
 * The console is handed its token once, in the url the server prints. It keeps
 * it and drops it out of the address bar so a shared screenshot is not a key.
 */
function claim(): string | null {
  const asked = new URLSearchParams(location.search).get('token');
  if (asked) {
    localStorage.setItem(TOKEN_KEY, asked);
    const clean = new URL(location.href);
    clean.searchParams.delete('token');
    history.replaceState(null, '', clean.toString());
    return asked;
  }
  return localStorage.getItem(TOKEN_KEY);
}

let token = claim();

export function keepToken(value: string): void {
  localStorage.setItem(TOKEN_KEY, value);
  token = value;
}

function headers(extra?: Record<string, string>): Record<string, string> {
  return token ? { ...extra, authorization: `Bearer ${token}` } : { ...extra };
}

export class ConsoleError extends Error {
  readonly kind: string;

  constructor(kind: string, reason: string) {
    super(reason);
    this.name = 'ConsoleError';
    this.kind = kind;
  }
}

/** A refused op carries the same `{ kind, reason }` the tauri bridge rejects with. */
async function refusal(response: Response): Promise<ConsoleError> {
  if (response.status === 401) return new ConsoleError('unauthorized', 'this console is not carrying a valid token');

  const body: unknown = await response.json().catch(() => null);
  if (body && typeof body === 'object') {
    const kind = (body as { kind?: unknown }).kind;
    return new ConsoleError(typeof kind === 'string' ? kind : 'host', describeError(body));
  }
  return new ConsoleError('host', `${response.status} ${response.statusText}`);
}

/** The http twin of the desktop shell's tauri `invoke`. */
export async function call<T>(op: string, params: Record<string, unknown> = {}): Promise<T> {
  const response = await fetch(`/api/rpc/${op}`, {
    method: 'POST',
    headers: headers({ 'content-type': 'application/json' }),
    body: JSON.stringify(params),
  });
  if (!response.ok) throw await refusal(response);
  return (await response.json()) as T;
}

/** Hands a file the browser holds to the server, which answers with its path. */
export async function upload(file: File): Promise<string> {
  const response = await fetch(`/api/artifact?name=${encodeURIComponent(file.name)}`, {
    method: 'POST',
    headers: headers({ 'content-type': 'application/octet-stream' }),
    body: file,
  });
  if (!response.ok) throw await refusal(response);
  const { path } = (await response.json()) as { path: string };
  return path;
}

export function eventsUrl(): string {
  const url = new URL('/api/events', location.href);
  url.protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
  if (token) url.searchParams.set('token', token);
  return url.toString();
}
