#!/usr/bin/env bash

start_timestamp=""

# set up start timestamp
reset_timestamp() {
    start_timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
}

# before all tests
setup() {
    reset_timestamp

    echo "Building docker compose..." >&3
    docker builder prune --all --force
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

    echo "Waiting: $container | $level $target > $logline" >&3
    
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

    echo "Waiting: $container | $logline" >&3
    
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

start_container() {
    local container=$1

    echo "Starting container: $container" >&3
    docker compose --project-directory $project start $container
}

stop_container() {
    local container=$1

    echo "Stopping container: $container" >&3
    docker compose --project-directory $project stop $container
}

restart_container() {
    local container=$1

    echo "Restarting container: $container" >&3
    docker compose --project-directory $project restart $container
}
