# build this app
FROM rust:1.81 as app-builder
RUN rustup install nightly
RUN rustup default nightly
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates
RUN apt-get update && apt-get install -y pkg-config libssl-dev
WORKDIR /usr/src/bot
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --target x86_64-unknown-linux-musl --release --locked

# final image
FROM scratch

# Copy the compiled binary from the lazymc-docker-proxy stage
COPY --from=app-builder /usr/src/bot/target/x86_64-unknown-linux-musl/release/bot /usr/local/bin/bot

# Set the working directory
WORKDIR /app

# Run lazymc by default
ENTRYPOINT ["bot"]
