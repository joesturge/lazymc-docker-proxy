# Use an official Rust image as the base
FROM rust:1.74 as builder

# Install dependencies for compiling lazymc
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Set the working directory
WORKDIR /usr/src/lazymc

# Clone the lazymc repository and compile the binary
ARG LAZYMC_VERSION=v0.2.11

RUN git clone --branch $LAZYMC_VERSION https://github.com/timvisee/lazymc . && \
    cargo build --release

# Use an official Eclipse Temurin image as the base
FROM eclipse-temurin:19-jre-jammy

# Install docker
RUN apt-get update && apt-get install -y docker.io

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/lazymc/target/release/lazymc /usr/local/bin/lazymc

# Copy the configuration file
COPY lazymc.toml /minecraft/lazymc.toml

# Copy the configuration file
COPY entrypoint.sh /minecraft/entrypoint.sh
COPY start-command.sh /minecraft/start-command.sh

# Set the working directory
WORKDIR /minecraft

# Run lazymc by default
ENTRYPOINT ["/minecraft/entrypoint.sh"]