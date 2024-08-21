# setup lazymc versions
ARG LAZYMC_VERSION=0.2.11
ARG LAZYMC_LEGACY_VERSION=0.2.10

# build lazymc
FROM rust:1.80 as lazymc-builder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
RUN apt-get update && apt-get install -y pkg-config libssl-dev
WORKDIR /usr/src/lazymc
ARG LAZYMC_VERSION
ENV LAZYMC_VERSION=$LAZYMC_VERSION
RUN git clone --branch v$LAZYMC_VERSION https://github.com/timvisee/lazymc .
RUN cargo build --target x86_64-unknown-linux-musl --release --locked

# build lazymc-legacy
FROM rust:1.80 as lazymc-legacy-builder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
RUN apt-get update && apt-get install -y pkg-config libssl-dev
WORKDIR /usr/src/lazymc
ARG LAZYMC_LEGACY_VERSION
ENV LAZYMC_LEGACY_VERSION=$LAZYMC_LEGACY_VERSION
RUN git clone --branch v$LAZYMC_LEGACY_VERSION https://github.com/timvisee/lazymc .
RUN cargo build --target x86_64-unknown-linux-musl --release --locked

# build this app
FROM rust:1.80 as app-builder
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
RUN apt-get update && apt-get install -y pkg-config libssl-dev
WORKDIR /usr/src/lazymc-docker-proxy
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --target x86_64-unknown-linux-musl --release --locked

# health init
FROM busybox:1.36.1-uclibc as health-init
RUN mkdir -p /app && echo "STARTING" > /app/health

# final image
FROM scratch

# setup lazymc version
ARG LAZYMC_VERSION
ENV LAZYMC_VERSION=$LAZYMC_VERSION
ARG LAZYMC_LEGACY_VERSION
ENV LAZYMC_LEGACY_VERSION=$LAZYMC_LEGACY_VERSION

# Copy the compiled binary from the lazymc-builder stage
COPY --from=lazymc-builder /usr/src/lazymc/target/x86_64-unknown-linux-musl/release/lazymc /usr/local/bin/lazymc

# Copy the compiled binary from the lazymc-legacy-builder stage
COPY --from=lazymc-legacy-builder /usr/src/lazymc/target/x86_64-unknown-linux-musl/release/lazymc /usr/local/bin/lazymc-legacy

# Copy the compiled binary from the lazymc-docker-proxy stage
COPY --from=app-builder /usr/src/lazymc-docker-proxy/target/x86_64-unknown-linux-musl/release/lazymc-docker-proxy /usr/local/bin/lazymc-docker-proxy

# Copy the health init state
COPY --from=health-init /app/health /app/health

# Set the working directory
WORKDIR /app

# Set the healthcheck
HEALTHCHECK --start-period=1m --interval=5s --retries=24 CMD ["lazymc-docker-proxy", "--health"]

# Run lazymc by default
ENTRYPOINT ["lazymc-docker-proxy"]
