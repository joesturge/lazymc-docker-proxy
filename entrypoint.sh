#!/bin/bash

set -e

echo "Setting up configuration file..."

# Path to the lazymc.toml file
file_path="lazymc.toml"

# Default value for FORGE
[ -z "$DEBUG" ] && DEBUG=false
[ -z "$FORGE" ] && FORGE=false
[ -z "$MINIMUM_ONLINE_TIME" ] && MINIMUM_ONLINE_TIME=30
[ -z "$PROBE_ON_START" ] && PROBE_ON_START=false
[ -z "$RCON_ENABLED" ] && RCON_ENABLED="false"
[ -z "$RCON_PASSWORD" ] && RCON_PASSWORD=""
[ -z "$RCON_PORT" ] && RCON_PORT=25575
[ -z "$SEND_PROXY_V2" ] && SEND_PROXY_V2=false
[ -z "$SLEEP_AFTER" ] && SLEEP_AFTER=60

# If debug set RUST_LOG=trace
if [ "$DEBUG" = "true" ]; then
    export RUST_LOG=trace
fi

# Known placeholders to replace
placeholders=(
    "FORGE"
    "MINIMUM_ONLINE_TIME"
    "PROBE_ON_START"
    "RCON_ENABLED"
    "RCON_PASSWORD"
    "RCON_PORT"
    "SEND_PROXY_V2"
    "SERVER_HOST"
    "SERVER_PORT"
    "SLEEP_AFTER"
)

# Loop through placeholders and replace them with environment variable values
for placeholder in "${placeholders[@]}"; do
    if [ ! -z "${!placeholder}" ]; then  # Check if the environment variable is set
        sed -i "s/\${${placeholder}}/${!placeholder}/g" "$file_path"
    else
        echo "Warning: Environment variable $placeholder is not set."
    fi
done

echo "Configuration file has been set up."

lazymc start
