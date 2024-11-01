#!/usr/bin/env bash

set -euo pipefail

TARGET_PLATFORM=${TARGETPLATFORM}

case ${TARGET_PLATFORM} in
  "linux/amd64")
    RUST_TARGET=x86_64-unknown-linux-musl
    RUST_LINKER=x86_64-linux-gnu-gcc
    ;;
  "linux/arm64")
    RUST_TARGET=aarch64-unknown-linux-musl
    RUST_LINKER=aarch64-linux-gnu-gcc
    ;;
  *)
    echo "Unsupported platform: ${TARGET_PLATFORM}"
    exit 1
    ;;
esac

rustup target add $RUST_TARGET
cargo build \
  --config target.$RUST_TARGET.linker=\"$RUST_LINKER\" \
  --release \
  --locked \
  --target $RUST_TARGET

mv target/$RUST_TARGET/release/tf_plan_format /tf_plan_format
