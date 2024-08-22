#!/usr/bin/env bats

load ../util.bash

project="./tests/bats/vanilla"

@test "Vanilla - Test lazymc stops server when idle" {
    # restart the lazymc container
    restart_container lazymc-vanilla

    # reset the start timestamp
    reset_timestamp

    #
    # The above steps ensures that the containers are in a clean state
    #

    # wait for lazymc process to start
    wait_for_formatted_log "lazymc-vanilla" "INFO" "lazymc-docker-proxy::entrypoint" "Starting lazymc process for group: mc..."

    # wait for lazymc to start the server
    wait_for_formatted_log "lazymc-vanilla" "INFO" "mc::lazymc" "Starting server..."

    # wait for the server to be online
    wait_for_formatted_log "lazymc-vanilla" "INFO" "mc::lazymc::monitor" "Server is now online" 300

    # wait for the mincraft server to be ready
    wait_for_log "mc-vanilla" "RCON running on 0.0.0.0:25575" 300

    # wait for the server to be idle
    wait_for_formatted_log "lazymc-vanilla" "INFO" "mc::lazymc::monitor" "Server has been idle, sleeping..." 120

    # wait for the server to be stopped
    wait_for_formatted_log "lazymc-vanilla" "INFO" "mc::lazymc-docker-proxy::command" "Received SIGTERM, stopping server..."

    # wait for the server to exit
    wait_for_log "mc-vanilla" "Thread RCON Listener stopped"

    # wait for lazymc to sleep
    wait_for_formatted_log "lazymc-vanilla" "INFO" "mc::lazymc::monitor" "Server is now sleeping"
}
