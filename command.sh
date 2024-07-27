#!/bin/bash

# Start the lazymc-docker-proxy
echo "Starting lazymc-docker-proxy..."
lazymc-docker-proxy start

# Function to handle SIGTERM signal
handle_sigterm() {
    echo "Received SIGTERM signal"
    echo "Stopping lazymc-docker-proxy..."
    lazymc-docker-proxy stop
    exit 0
}

# trap sigterm and stop all containers
trap handle_sigterm TERM INT

# wait for sigterm
while true; do
  sleep 1
done