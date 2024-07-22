#!/bin/bash

# check $LAZYMC_GROUP is set
if [ -z "$LAZYMC_GROUP" ]; then
  echo "Error: Environment variable LAZYMC_GROUP is not set."
  exit 1
fi

# Get IDs of stopped containers with the label and filters
stopped_containers=$(docker ps -a -q --filter "label=lazymc.group=${LAZYMC_GROUP}" --filter "status=exited")

# Start stopped containers if any
if [ -n "$stopped_containers" ]; then
  echo "Starting stopped containers..."
  docker start $stopped_containers
fi

# Get IDs of all containers with the label and filter, including running ones
all_containers=$(docker ps -q --filter "label=lazymc.group=${LAZYMC_GROUP}")

if [ -z "$all_containers" ]; then
  echo "Error: No containers found with label lazymc.group=${LAZYMC_GROUP}."
  exit 1
fi

# Function to handle SIGTERM signal
handle_sigterm() {
  echo "SIGTERM received. Stopping all containers..."
  docker stop $all_containers
  echo "Stopped all containers."
  exit 0
}

# trap sigterm and stop all containers
trap 'handle_sigterm' SIGTERM

# wait for sigterm
while true; do
  sleep 1
done