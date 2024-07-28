#!/usr/bin/env bats

# wait for a log line to appear in a container's logs
wait_for_log() {
    local container=$1
    local logline=$2
    local timeout=${3:-60}

    timeout ${timeout}s bash -c "until docker-compose logs --no-color ${container} | grep -q '${logline}'; do sleep 1; done"
}

# before all tests
setup() {
    echo "Building docker-compose..." >&3
    docker-compose build

    echo "Starting docker-compose..." >&3
    docker-compose up -d
}

# after all tests
teardown() {
    echo "Stopping docker-compose..." >&3
    docker-compose down
}

@test "Test lazymc stops server when idle" {
    # wait for lazymc process to start
    echo "Waiting for lazymc process to start..." >&3
    wait_for_log "lazymc" "INFO  lazymc-docker-proxy::entrypoint > Starting lazymc process..." 300

    # wait for lazymc to start the server
    echo "Waiting for server to start..." >&3
    wait_for_log "lazymc" "INFO  lazymc > Starting server..."

    # wait for the server to be online
    echo "Waiting for server to be online..." >&3
    wait_for_log "lazymc" "INFO  lazymc::monitor > Server is now online"

    # wait for the mincraft server to be ready
    echo "Waiting for minecraft server to be ready..." >&3
    wait_for_log "mc" "RCON running on 0.0.0.0:25575" 300

    # wait for the server to be idle
    echo "Waiting for server to be idle..." >&3
    wait_for_log "lazymc" "INFO  lazymc::monitor > Server has been idle, sleeping..." 120

    # wait for the server to be stopped
    echo "Waiting for stop command..." >&3
    wait_for_log "lazymc" "INFO  lazymc-docker-proxy::command > Received SIGTERM, stopping server..."

    # wait for the server to exit
    echo "Waiting for server to exit..." >&3
    wait_for_log "mc" "Thread RCON Listener stopped"

    # wait for lazymc to sleep
    echo "Server is now sleeping..." >&3
    wait_for_log "lazymc" "INFO  lazymc::monitor > Server is now sleeping"
}
