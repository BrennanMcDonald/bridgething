#!/usr/bin/env bash
# Takes a machine from nothing to a running bridgething console: toolchains,
# build, systemd unit, started service. Safe to run again; every stage checks
# what is already there before doing anything.
#
#   headless/scripts/install.sh                 # the whole way
#   headless/scripts/install.sh --no-service    # build only, leave systemd alone
#   headless/scripts/install.sh --features voice
set -euo pipefail

PREFIX=/opt/bridgething-console
SERVICE=bridgething-console
USER_NAME=bridgething
BIND=0.0.0.0:8899
FEATURES=
BINARY=
WEB=
WITH_SERVICE=1
WITH_DEPS=1
WITH_BUILD=1
ASSUME_YES=0

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
here=$repo/headless

usage() {
  cat <<'USAGE'
usage: headless/scripts/install.sh [options]

  --prefix DIR      where the binary and console land (default /opt/bridgething-console)
  --service NAME    systemd unit name, and the state directory under /var/lib (default bridgething-console)
  --user NAME       the unprivileged user the service runs as (default bridgething)
  --bind ADDR:PORT  what the console listens on (default 0.0.0.0:8899)
  --features LIST   extra cargo features, comma separated (voice, desktop-session)
  --binary PATH     install this binary instead of the one in target/release
  --web DIR         install these console assets instead of headless/dist
  --no-deps         do not install toolchains; fail if they are missing
  --no-build        install whatever is already built
  --no-service      build only; do not touch systemd
  -y, --yes         do not ask before installing toolchains or packages
  -h, --help        this
USAGE
}

while [[ $# -gt 0 ]]; do
  case $1 in
    --prefix) PREFIX=$2; shift 2 ;;
    --service) SERVICE=$2; shift 2 ;;
    --user) USER_NAME=$2; shift 2 ;;
    --bind) BIND=$2; shift 2 ;;
    --features) FEATURES=$2; shift 2 ;;
    --binary) BINARY=$2; shift 2 ;;
    --web) WEB=$2; shift 2 ;;
    --no-deps) WITH_DEPS=0; shift ;;
    --no-build) WITH_BUILD=0; shift ;;
    --no-service) WITH_SERVICE=0; shift ;;
    -y|--yes) ASSUME_YES=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

say() { printf '\n==> %s\n' "$*"; }
note() { printf '    %s\n' "$*"; }
die() { printf '\nerror: %s\n' "$*" >&2; exit 1; }

confirm() {
  [[ $ASSUME_YES == 1 ]] && return 0
  read -r -p "    $1 [Y/n] " answer </dev/tty || return 1
  [[ -z $answer || $answer == [yY]* ]]
}

# sudo only where it is actually needed, and only if we are not already root
if [[ $(id -u) == 0 ]]; then
  sudo() { "$@"; }
elif ! command -v sudo >/dev/null; then
  die "this needs root for the install stage and sudo is not here; run it as root or pass --no-service"
fi

# at least this rustc, from the workspace manifest
msrv=$(sed -n 's/^rust-version = "\([^"]*\)".*/\1/p' "$repo/Cargo.toml" | head -1)
newest() { printf '%s\n%s\n' "$1" "$2" | sort -V | tail -1; }

# MARK: toolchains

ensure_packages() {
  command -v apt-get >/dev/null || {
    note "not a debian box; make sure a c compiler, curl, git, and unzip are installed"
    return 0
  }

  local wanted=(build-essential curl ca-certificates git unzip pkg-config)
  if [[ $FEATURES == *voice* ]]; then
    wanted+=(cmake clang libclang-dev)
  fi

  local missing=()
  for package in "${wanted[@]}"; do
    dpkg-query -W -f='${Status}' "$package" 2>/dev/null | grep -q '^install ok installed$' || missing+=("$package")
  done
  if [[ ${#missing[@]} == 0 ]]; then
    note "the build's apt packages are already here"
    return 0
  fi

  say "apt packages the build needs: ${missing[*]}"
  confirm "install them?" || die "cannot build without ${missing[*]}"
  sudo apt-get update
  sudo apt-get install -y "${missing[@]}"
}

ensure_rust() {
  if ! command -v rustup >/dev/null && ! command -v cargo >/dev/null; then
    say "rust is not here"
    confirm "install it with rustup?" || die "cannot build without cargo"
    curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi

  # rustup can be installed with no toolchain in it at all, which is what a
  # distro package leaves behind
  if command -v rustup >/dev/null && ! rustup show active-toolchain >/dev/null 2>&1; then
    say "rustup has no toolchain installed"
    confirm "install stable and make it the default?" || die "cannot build without a toolchain"
    rustup toolchain install stable
    rustup default stable
  fi

  command -v cargo >/dev/null || die "cargo is still not on PATH; open a new shell and run this again"

  local have
  have=$(rustc --version | awk '{print $2}')
  if [[ -n $msrv && $(newest "$have" "$msrv") != "$have" ]]; then
    note "rustc $have is older than the $msrv this workspace needs"
    if command -v rustup >/dev/null; then
      confirm "update the stable toolchain?" || die "cannot build with rustc $have"
      rustup update stable
      rustup default stable
    else
      die "update rust to $msrv or newer"
    fi
  fi
  note "rustc $(rustc --version | awk '{print $2}')"
}

ensure_bun() {
  if ! command -v bun >/dev/null && [[ -x $HOME/.bun/bin/bun ]]; then
    export PATH="$HOME/.bun/bin:$PATH"
  fi
  if ! command -v bun >/dev/null; then
    say "bun is not here"
    confirm "install it?" || die "cannot build the console without bun"
    curl -fsSL https://bun.sh/install | bash
    export PATH="$HOME/.bun/bin:$PATH"
  fi
  command -v bun >/dev/null || die "bun is still not on PATH; open a new shell and run this again"
  note "bun $(bun --version)"
}

# MARK: build

build() {
  local flags=(--release -p bridgething-headless)
  [[ -n $FEATURES ]] && flags+=(--features "$FEATURES")

  say "building the host${FEATURES:+ with $FEATURES}"
  (cd "$repo" && cargo build "${flags[@]}")

  # through turbo, so the workspace packages the console imports are built first
  say "building the console"
  (cd "$repo" && bun install)
  (cd "$repo" && bun run build --filter=@bridgething/headless-frontend)
}

# MARK: install

install_service() {
  local binary=${BINARY:-$repo/target/release/bridgething-headless}
  local web=${WEB:-$here/dist}
  [[ -x $binary ]] || die "no binary at $binary; build it, or point --binary at one"
  [[ -f $web/index.html ]] || die "no console at $web; build it, or point --web at one"

  [[ -d /run/systemd/system ]] || die "this box is not running systemd; start the binary yourself: $binary"

  say "installing to $PREFIX"
  id -u "$USER_NAME" >/dev/null 2>&1 ||
    sudo useradd --system --no-create-home --shell /usr/sbin/nologin "$USER_NAME"

  sudo install -d -m 0755 "$PREFIX"
  sudo install -m 0755 "$binary" "$PREFIX/bridgething-headless"
  sudo rm -rf "$PREFIX/web"
  sudo cp -r "$web" "$PREFIX/web"
  sudo chown -R root:root "$PREFIX"

  say "installing $SERVICE.service"
  sed -e "s|@PREFIX@|$PREFIX|g" -e "s|@SERVICE@|$SERVICE|g" -e "s|@USER@|$USER_NAME|g" -e "s|@BIND@|$BIND|g" \
    "$here/systemd/bridgething-console.service.in" |
    sudo tee "/etc/systemd/system/$SERVICE.service" >/dev/null
  sudo chmod 0644 "/etc/systemd/system/$SERVICE.service"

  sudo systemctl daemon-reload
  sudo systemctl enable --now "$SERVICE"
  sudo systemctl restart "$SERVICE"
}

report() {
  # the token is minted on the first start, so give the service a moment for it
  local token= at=0
  while [[ $at -lt 20 ]]; do
    token=$(sudo cat "/var/lib/$SERVICE/config/console-token" 2>/dev/null || true)
    if [[ -n $token ]]; then
      break
    fi
    sleep 0.25
    at=$((at + 1))
  done

  local address port
  address=$(hostname -I 2>/dev/null | awk '{print $1}')
  port=${BIND##*:}

  if ! sudo systemctl is-active --quiet "$SERVICE"; then
    printf '\n%s did not stay up. what it said:\n\n' "$SERVICE"
    sudo journalctl -u "$SERVICE" -n 30 --no-pager
    exit 1
  fi

  say "the console is up"
  if [[ -n $token ]]; then
    note "open it once with its token: http://${address:-<this-machine>}:$port/?token=$token"
    note "the page keeps the token; later visits are just http://${address:-<this-machine>}:$port/"
  else
    note "http://${address:-<this-machine>}:$port/"
  fi
  note "logs:    journalctl -u $SERVICE -f"
  note "restart: sudo systemctl restart $SERVICE"
}

if [[ $WITH_DEPS == 1 ]]; then
  ensure_packages
  ensure_rust
  if [[ $WITH_BUILD == 1 ]]; then
    ensure_bun
  fi
fi

if [[ $WITH_BUILD == 1 ]]; then
  build
fi

if [[ $WITH_SERVICE == 1 ]]; then
  install_service
  report
else
  say "built, and systemd left alone"
  note "run it: $repo/target/release/bridgething-headless --web-root $here/dist"
fi

if [[ -x $HOME/.bun/bin/bun && -z $(command -v bun 2>/dev/null || true) ]]; then
  note "bun landed in ~/.bun/bin; open a new shell before using it by hand"
fi
