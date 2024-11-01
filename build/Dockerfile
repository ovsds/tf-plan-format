FROM --platform=$BUILDPLATFORM base_build_image AS base
WORKDIR /app
COPY --from=sources Cargo.toml Cargo.lock ./
COPY --from=sources src/ ./src/

RUN apt-get update && \
    apt-get install -y \
      g++-x86-64-linux-gnu \
      gcc-aarch64-linux-gnu
RUN curl -L "https://github.com/roblabla/MacOSX-SDKs/releases/download/13.3/MacOSX13.3.sdk.tar.xz" | tar -J -x -C /opt
ENV SDKROOT=/opt/MacOSX13.3.sdk

FROM base AS build
ARG RUST_TARGET
ARG RUST_LINKER

RUN rustup target add $RUST_TARGET
RUN cargo build \
  --config target.$RUST_TARGET.linker=\"$RUST_LINKER\" \
  --release \
  --locked \
  --target $RUST_TARGET

FROM scratch AS bin
ARG RUST_TARGET
COPY --from=build /app/target/$RUST_TARGET/release/tf_plan_format /tf_plan_format

FROM base AS build-static
ARG TARGETPLATFORM
COPY --from=sources build/build_static.sh ./
RUN ./build_static.sh

FROM scratch AS image
COPY --from=build-static /tf_plan_format /tf_plan_format
ENTRYPOINT ["/tf_plan_format"]
