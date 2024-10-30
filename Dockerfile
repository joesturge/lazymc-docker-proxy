# setup lazymc versions
ARG LAZYMC_VERSION=0.2.11
ARG LAZYMC_LEGACY_VERSION=0.2.10

# build lazymc
FROM --platform=$BUILDPLATFORM rust:1.82 as lazymc-builder
ARG TARGETARCH
ARG RUST_ARCH_AMD=${TARGETARCH/amd64/x86_64-unknown-linux-musl}
ARG RUST_ARCH=${RUST_ARCH_AMD/arm64/aarch64-unknown-linux-musl}
RUN rustup target add $RUST_ARCH
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
RUN apt-get update && apt-get install -y pkg-config libssl-dev crossbuild-essential-arm64 crossbuild-essential-armhf
WORKDIR /usr/src/lazymc
ARG LAZYMC_VERSION
ENV LAZYMC_VERSION=$LAZYMC_VERSION
RUN git clone --branch v$LAZYMC_VERSION https://github.com/timvisee/lazymc .
RUN <<EOF
  mkdir -p ./.cargo/
  echo [target.aarch64-unknown-linux-musl] >> ./.cargo/config.toml
  echo linker = \"aarch64-linux-gnu-gcc\" >> ./.cargo/config.toml
EOF
RUN cargo build --target $RUST_ARCH --release --locked
RUN mv /usr/src/lazymc/target/$RUST_ARCH /usr/src/lazymc/target/output_final

# build lazymc-legacy
FROM --platform=$BUILDPLATFORM rust:1.82 as lazymc-legacy-builder
ARG TARGETARCH
ARG RUST_ARCH_AMD=${TARGETARCH/amd64/x86_64-unknown-linux-musl}
ARG RUST_ARCH=${RUST_ARCH_AMD/arm64/aarch64-unknown-linux-musl}
RUN rustup target add $RUST_ARCH
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
RUN apt-get update && apt-get install -y pkg-config libssl-dev crossbuild-essential-arm64 crossbuild-essential-armhf
WORKDIR /usr/src/lazymc
ARG LAZYMC_LEGACY_VERSION
ENV LAZYMC_LEGACY_VERSION=$LAZYMC_LEGACY_VERSION
RUN git clone --branch v$LAZYMC_LEGACY_VERSION https://github.com/timvisee/lazymc .
RUN <<EOF
  mkdir -p ./.cargo/
  echo [target.aarch64-unknown-linux-musl] >> ./.cargo/config.toml
  echo linker = \"aarch64-linux-gnu-gcc\" >> ./.cargo/config.toml
EOF
RUN cargo build --target $RUST_ARCH --release --locked
RUN mv /usr/src/lazymc/target/$RUST_ARCH /usr/src/lazymc/target/output_final

# build this app
FROM --platform=$BUILDPLATFORM rust:1.82 as app-builder
ARG TARGETARCH
ARG RUST_ARCH_AMD=${TARGETARCH/amd64/x86_64-unknown-linux-musl}
ARG RUST_ARCH=${RUST_ARCH_AMD/arm64/aarch64-unknown-linux-musl}
RUN rustup target add $RUST_ARCH
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
RUN apt-get update && apt-get install -y pkg-config libssl-dev crossbuild-essential-arm64 crossbuild-essential-armhf
WORKDIR /usr/src/lazymc-docker-proxy
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN <<EOF
  mkdir -p ./.cargo/
  echo [target.aarch64-unknown-linux-musl] >> ./.cargo/config.toml
  echo linker = \"aarch64-linux-gnu-gcc\" >> ./.cargo/config.toml
EOF
RUN cargo build --target $RUST_ARCH --release --locked
RUN mv /usr/src/lazymc-docker-proxy/target/$RUST_ARCH /usr/src/lazymc-docker-proxy/target/output_final

# health init
FROM --platform=$BUILDPLATFORM busybox:1.37.0-uclibc as health-init
RUN mkdir -p /app && echo "STARTING" > /app/health

# final image
FROM scratch

# setup lazymc version
ARG LAZYMC_VERSION
ENV LAZYMC_VERSION=$LAZYMC_VERSION
ARG LAZYMC_LEGACY_VERSION
ENV LAZYMC_LEGACY_VERSION=$LAZYMC_LEGACY_VERSION

# Copy the compiled binary from the lazymc-builder stage
COPY --from=lazymc-builder /usr/src/lazymc/target/output_final/release/lazymc /usr/local/bin/lazymc

# Copy the compiled binary from the lazymc-legacy-builder stage
COPY --from=lazymc-legacy-builder /usr/src/lazymc/target/output_final/release/lazymc /usr/local/bin/lazymc-legacy

# Copy the compiled binary from the lazymc-docker-proxy stage
COPY --from=app-builder /usr/src/lazymc-docker-proxy/target/output_final/release/lazymc-docker-proxy /usr/local/bin/lazymc-docker-proxy

# Copy the health init state
COPY --from=health-init /app/health /app/health

# Set the working directory
WORKDIR /app

# Set the healthcheck
HEALTHCHECK --start-period=1m --interval=5s --retries=24 CMD ["lazymc-docker-proxy", "--health"]

# Run lazymc by default
ENTRYPOINT ["lazymc-docker-proxy"]
