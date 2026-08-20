#!/usr/bin/env bash
# Puts the headless console on a raspberry pi (or any systemd box) and starts it.
# Run it from a checkout that has already built both halves:
#
#   cargo build --release -p bridgething-headless
#   cd headless && bun run build
#
# or point it at an unpacked release with BINARY= and WEB=.
set -euo pipefail

PREFIX=${PREFIX:-/opt/bridgething-console}
SERVICE=${SERVICE:-bridgething-console}
USER_NAME=${USER_NAME:-bridgething}
here=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

BINARY=${BINARY:-$here/../target/release/bridgething-headless}
WEB=${WEB:-$here/dist}

if [[ ! -x $BINARY ]]; then
  echo "no binary at $BINARY - run: cargo build --release -p bridgething-headless" >&2
  exit 1
fi
if [[ ! -f $WEB/index.html ]]; then
  echo "no console assets at $WEB - run: cd headless && bun run build" >&2
  exit 1
fi

sudo id -u "$USER_NAME" >/dev/null 2>&1 || sudo useradd --system --no-create-home --shell /usr/sbin/nologin "$USER_NAME"

sudo install -d -m 0755 "$PREFIX"
sudo install -m 0755 "$BINARY" "$PREFIX/bridgething-headless"
sudo rm -rf "$PREFIX/web"
sudo cp -r "$WEB" "$PREFIX/web"
sudo chown -R root:root "$PREFIX"

sudo install -m 0644 "$here/systemd/$SERVICE.service" "/etc/systemd/system/$SERVICE.service"
sudo systemctl daemon-reload
sudo systemctl enable --now "$SERVICE"

sleep 2
token=$(sudo cat /var/lib/bridgething-console/config/console-token 2>/dev/null || true)
address=$(hostname -I | awk '{print $1}')
echo
echo "the console is up on http://${address:-<this-machine>}:8899/"
[[ -n $token ]] && echo "open it once with the token: http://${address:-<this-machine>}:8899/?token=$token"
echo "logs: journalctl -u $SERVICE -f"
