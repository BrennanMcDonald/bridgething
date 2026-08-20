# headless

The bridgething companion host for a machine with no desktop - a Raspberry Pi
on a shelf, a NUC in a closet, anything with systemd. It holds the same link to
a Car Thing the tray app holds, and instead of a window it serves the console
over http.

```
headless/
  src-server/     the rust binary (bridgething-headless)
  src/            the console entrypoint: http session, browser host adapter
  systemd/        the unit template an install renders
  scripts/        install.sh, which goes from a bare machine to a running service
```

Both halves of the console are shared with the desktop app and neither is
reimplemented here:

- **the ops** live in `crates/host-shell` (`bridgething-host-shell`). The tray
  app wraps them in `#[tauri::command]`; this server routes to them from
  `POST /api/rpc/{op}`. There is one implementation of "install this webapp".
- **the screens** live in `packages/console` (`@bridgething/console`). The two
  entrypoints differ only in how they reach the host: `TauriSession` +
  `tauriHost` on the desktop, `HttpSession` + `webHost` here.

## Running it

Build both halves, then start the server:

```sh
just headless-build          # cargo build --release -p bridgething-headless && bun run build
just headless-serve          # or: cargo run -p bridgething-headless
```

It prints the url to open, with the token in the query. Open it once; the page
keeps the token and drops it out of the address bar.

For frontend iteration, run the server and vite side by side - vite proxies
`/api` (websocket included) at `127.0.0.1:8899`:

```sh
just headless-serve          # one terminal
just headless-dev            # the other, on :1421
```

## Installing on a pi

```sh
git clone https://github.com/JoeyEamigh/bridgething && cd bridgething
headless/scripts/install.sh
```

That is the whole thing: it installs what the build needs (apt packages, a
rust toolchain at or above the workspace's msrv, bun), builds both halves,
renders the systemd unit, starts it, and prints the url with its token. It
asks before installing anything; `-y` says yes to all of it. Running it again
is how you upgrade.

The binary and the console land in `/opt/bridgething-console`, state under
`/var/lib/bridgething-console`.

| flag | what it changes |
| --- | --- |
| `--prefix DIR` | where the binary and console land |
| `--service NAME` | the unit name, and the state directory under `/var/lib` |
| `--user NAME` | the unprivileged user the service runs as |
| `--bind ADDR:PORT` | what the console listens on |
| `--features LIST` | extra cargo features |
| `--binary`, `--web` | install artifacts from somewhere other than the build |
| `--no-deps`, `--no-build`, `--no-service` | skip a stage |
| `-y` | do not ask |

The default build leaves the voice stack out, because whisper and onnxruntime
are a long build and a lot of memory for a pi. Turn them on with
`--features voice`; turn on geoclue, freedesktop notifications, and
speech-dispatcher with `--features desktop-session` if the box runs a session
that offers them.

### Building somewhere else

A pi is a slow place to compile rust, and `bun install` pulls the whole
monorepo's node_modules. Build on a bigger machine, copy the two artifacts
over, and let the pi do only the install:

```sh
# on the build machine
cargo build --release --target aarch64-unknown-linux-gnu -p bridgething-headless
cd headless && bun run build

# on the pi, in a checkout
headless/scripts/install.sh --no-deps --no-build \
  --binary ~/bridgething-headless --web ~/dist
```

## Flags

| flag | default | what it does |
| --- | --- | --- |
| `--bind` | `0.0.0.0:8899` | where the console listens |
| `--gateway-url` | `ws://127.0.0.1:8892/` | the daemon dialed when the console asks for a link by hand |
| `--data-dir` | the xdg directories | state, cache, and config under one root |
| `--web-root` | `web/` beside the binary | the built console assets |
| `--token` | minted and kept | the token the console has to present; keep it url-safe |
| `--no-auth` | off | serve to anyone who can reach the port |
| `--no-mdns` | off | stop answering mdns for this console |

Every flag also reads an environment variable (`BRIDGETHING_CONSOLE_BIND`,
`BRIDGETHING_GATEWAY_URL`, `BRIDGETHING_CONSOLE_DATA_DIR`,
`BRIDGETHING_CONSOLE_WEB`, `BRIDGETHING_CONSOLE_TOKEN`), which is how the
systemd unit configures it.

## The http surface

| route | what it is |
| --- | --- |
| `POST /api/rpc/{op}` | one op, named the way the tray app names it, arguments as a json object |
| `GET /api/events` | the invalidation socket: `{"name":"invalidate:peers","id":null}` |
| `POST /api/artifact?name=` | spools an uploaded artifact and answers with the path to push it from |
| everything else | the console, with an index.html fallback for client routes |

Everything under `/api` needs the token: `Authorization: Bearer <token>`, or
`?token=` on the websocket, which cannot carry a header.

## What the browser cannot do

Three things the tray app offers do not exist here, and the screens leave
themselves out rather than showing a dead control:

- **picking a file** is an upload. The picker hands the bytes to
  `POST /api/artifact` and the server answers with the path they landed at, so
  the push that follows is the same push the desktop makes. A path typed into
  the field by hand is read on the machine running the console, not on the one
  running the browser.
- **saving a file** is a download.
- **autostart, self-update, and quit** belong to systemd and the package
  manager, so the rows are not rendered.
