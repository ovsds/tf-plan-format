ARG TOOLCHAIN=bin

FROM --platform=$BUILDPLATFORM base_build_image AS base
WORKDIR /app

RUN apt-get update && \
    apt-get install -y \
      clang \
      g++-x86-64-linux-gnu \
      gcc-aarch64-linux-gnu
RUN curl -L "https://github.com/roblabla/MacOSX-SDKs/releases/download/13.3/MacOSX13.3.sdk.tar.xz" | tar -J -x -C /opt
ENV SDKROOT=/opt/MacOSX13.3.sdk
ENV CC_x86_64_apple_darwin=clang \
    CC_aarch64_apple_darwin=clang

FROM base AS toolchain-bin
ARG RUST_TARGET
ARG RUST_LINKER
ENV RUST_TARGET=$RUST_TARGET \
    RUST_LINKER=$RUST_LINKER

FROM base AS toolchain-static-amd64
ENV RUST_TARGET=x86_64-unknown-linux-musl \
    RUST_LINKER=x86_64-linux-gnu-gcc

FROM base AS toolchain-static-arm64
ENV RUST_TARGET=aarch64-unknown-linux-musl \
    RUST_LINKER=aarch64-linux-gnu-gcc

FROM toolchain-static-$TARGETARCH AS toolchain-static

FROM toolchain-$TOOLCHAIN AS build
RUN rustup target add $RUST_TARGET

COPY --from=sources Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    touch src/lib.rs && \
    cargo build \
      --config target.$RUST_TARGET.linker=\"$RUST_LINKER\" \
      --release \
      --locked \
      --target $RUST_TARGET && \
    rm -r src

COPY --from=sources src/ ./src/
RUN touch src/main.rs src/lib.rs && \
    cargo build \
      --config target.$RUST_TARGET.linker=\"$RUST_LINKER\" \
      --release \
      --locked \
      --target $RUST_TARGET && \
    mv target/$RUST_TARGET/release/tf_plan_format /tf_plan_format

FROM scratch AS bin
COPY --from=build /tf_plan_format /tf_plan_format

FROM scratch AS image
COPY --from=build /tf_plan_format /tf_plan_format
ENTRYPOINT ["/tf_plan_format"]
