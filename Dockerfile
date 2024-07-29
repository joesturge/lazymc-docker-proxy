# setup lazymc version
ARG LAZYMC_VERSION=0.2.11

# Use an official Rust image as the base
FROM rust:1.80 as lazymc-builder

# Install dependencies for compiling lazymc
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Set the working directory
WORKDIR /usr/src/lazymc

# Clone the lazymc repository and compile the binary
ARG LAZYMC_VERSION
ENV LAZYMC_VERSION=$LAZYMC_VERSION
RUN git clone --branch v$LAZYMC_VERSION https://github.com/timvisee/lazymc . && \
    cargo build --release --locked

# Use an official Rust image as the base
FROM rust:1.80 as app-builder

# Install dependencies for compiling lazymc-docker-proxy
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Set the working directory
WORKDIR /usr/src/lazymc-docker-proxy

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the binary
RUN cargo build --release --locked

# Use an official Eclipse Temurin image as the base
FROM eclipse-temurin:21-jre-jammy

# setup lazymc version
ARG LAZYMC_VERSION
ENV LAZYMC_VERSION=$LAZYMC_VERSION

# Install docker
RUN apt-get update && apt-get install -y docker.io

# Copy the compiled binary from the lazymc-builder stage
COPY --from=lazymc-builder /usr/src/lazymc/target/release/lazymc /usr/local/bin/lazymc

# Copy the compiled binary from the lazymc-docker-proxy stage
COPY --from=app-builder /usr/src/lazymc-docker-proxy/target/release/lazymc-docker-proxy /usr/local/bin/lazymc-docker-proxy

# Set the working directory
WORKDIR /app

# Run lazymc by default
ENTRYPOINT ["lazymc-docker-proxy"]