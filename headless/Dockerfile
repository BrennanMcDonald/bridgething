# syntax=docker/dockerfile:1
#
# The console, as an image a pi can run without a toolchain on it.
#
# Both build stages run on the machine doing the building and cross-compile to
# the target, because emulating a pi to build for a pi is the slow thing this
# exists to avoid. Build it from the repo root:
#
#   docker buildx build --platform linux/arm64 -f headless/Dockerfile -t bridgething-console .

# MARK: the console
#
# The bundle is the same whatever serves it, so this stage never cross-anything.
FROM --platform=$BUILDPLATFORM oven/bun:1 AS console
WORKDIR /src
COPY . .
# through turbo, so the workspace packages the console imports are built first
RUN bun install --frozen-lockfile && bun run build --filter=@bridgething/headless-frontend

# MARK: the server
FROM --platform=$BUILDPLATFORM rust:1.94-bookworm AS server
ARG TARGETARCH
WORKDIR /src

# aws-lc-sys comes in under rustls and is a cmake build, so the cross toolchain
# has to be a whole one, not just a linker
RUN <<TOOLCHAIN
set -eux
case "$TARGETARCH" in
  arm64) triple=aarch64-unknown-linux-gnu; cross="gcc-aarch64-linux-gnu libc6-dev-arm64-cross" ;;
  amd64) triple=x86_64-unknown-linux-gnu;  cross="gcc libc6-dev" ;;
  *) echo "no cross toolchain for $TARGETARCH" >&2; exit 1 ;;
esac
echo "$triple" > /triple
apt-get update
apt-get install --assume-yes --no-install-recommends cmake clang libclang-dev pkg-config $cross
rm -rf /var/lib/apt/lists/*
rustup target add "$triple"
TOOLCHAIN

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
  AR_aarch64_unknown_linux_gnu=aarch64-linux-gnu-ar \
  PKG_CONFIG_ALLOW_CROSS=1

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/src/target,sharing=locked \
  set -eux; \
  triple=$(cat /triple); \
  cargo build --release --locked -p bridgething-headless --target "$triple"; \
  cp "target/$triple/release/bridgething-headless" /bridgething-headless

# MARK: what actually runs
FROM debian:bookworm-slim AS runtime

# the catalog, the ota manifests, and the providers are all https
RUN apt-get update \
  && apt-get install --assume-yes --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && useradd --system --no-create-home --shell /usr/sbin/nologin bridgething

COPY --from=server /bridgething-headless /usr/local/bin/bridgething-headless
COPY --from=console /src/headless/dist /srv/console
RUN install -d -o bridgething -g bridgething /var/lib/bridgething-console

ENV BRIDGETHING_CONSOLE_WEB=/srv/console \
  BRIDGETHING_CONSOLE_DATA_DIR=/var/lib/bridgething-console \
  BRIDGETHING_CONSOLE_BIND=0.0.0.0:8899 \
  RUST_LOG=warn,bridgething_headless=info,bridgething_host_shell=info,bridgething_companion=info

USER bridgething
VOLUME /var/lib/bridgething-console
EXPOSE 8899
ENTRYPOINT ["/usr/local/bin/bridgething-headless"]
