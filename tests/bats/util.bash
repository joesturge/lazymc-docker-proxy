#!/usr/bin/env bash

start_timestamp=""

# before all tests
setup() {
    start_timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    echo "Building docker compose..." >&3
    docker compose --project-directory $project build

    echo "Starting docker compose..." >&3
    docker compose --project-directory $project up -d
}

# after all tests
teardown() {
    echo "Stopping docker compose..." >&3
    docker compose --project-directory $project down
}

# wait for a log line to appear in a container's logs
wait_for_formatted_log() {
    local container=$1
    local level=$2
    local target=$3
    local logline=$4
    local timeout=${5:-60}

    local regex="${level}\s+${target}\s+>\s+${logline}"
    
    trap 'exit 1' SIGINT SIGTERM
    until docker compose --project-directory $project logs --since $start_timestamp --no-color ${container} | grep -qE "$regex";
    do
        if [ $timeout -eq 0 ]; then
            echo "Timeout waiting for log: $logline" >&3
            exit 1
        fi
        sleep 1
        ((timeout--))
    done
}

# wait for a log line to appear in a container's logs
wait_for_log() {
    local container=$1
    local logline=$2
    local timeout=${3:-60}
    
    trap 'exit 1' SIGINT SIGTERM
    until docker compose --project-directory $project logs --since $start_timestamp --no-color ${container} | grep -q "$logline";
    do
        if [ $timeout -eq 0 ]; then
            echo "Timeout waiting for log: $logline" >&3
            exit 1
        fi
        sleep 1
        ((timeout--))
    done
}