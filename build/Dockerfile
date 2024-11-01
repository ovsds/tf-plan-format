FROM --platform=$BUILDPLATFORM base_build_image AS base
WORKDIR /app
COPY --from=sources Cargo.toml Cargo.lock ./
COPY --from=sources src/ ./src/

FROM base AS build
RUN cargo build --release

FROM scratch AS bin
COPY --from=build /app/target/release/tf_plan_format /tf_plan_format

FROM bin AS image
ENTRYPOINT ["/tf_plan_format"]

FROM base AS cross
RUN apt-get update && \
    apt-get install -y \
      g++-x86-64-linux-gnu \
      gcc-aarch64-linux-gnu
RUN curl -L "https://github.com/roblabla/MacOSX-SDKs/releases/download/13.3/MacOSX13.3.sdk.tar.xz" | tar -J -x -C /opt
ENV SDKROOT=/opt/MacOSX13.3.sdk

FROM cross AS cross_build
ARG RUST_TARGET
ARG RUST_LINKER

RUN rustup target add $RUST_TARGET
RUN cargo build \
  --config target.$RUST_TARGET.linker=\"$RUST_LINKER\" \
  --release \
  --target $RUST_TARGET

FROM scratch AS cross_bin
ARG RUST_TARGET
COPY --from=cross_build /app/target/$RUST_TARGET/release/tf_plan_format /tf_plan_format

FROM scratch AS cross_image
ARG TARGETOS
ARG TARGETARCH
COPY --from=binaries ./${TARGETOS}-${TARGETARCH}/tf_plan_format /tf_plan_format
ENTRYPOINT ["/tf_plan_format"]
