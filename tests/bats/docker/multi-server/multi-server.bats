#!/usr/bin/env bats

load ../util.bash

project="./tests/bats/docker/multi-server"

@test "Multiple Servers - Test primary and secondary servers stop when idle" {
    # restart the lazymc container
    restart_container lazymc-multi-server

    # reset the start timestamp
    reset_timestamp

    #
    # The above steps ensures that the containers are in a clean state
    #

    # wait for primary lazymc process to start
    wait_for_formatted_log "lazymc-multi-server" "INFO" "lazymc-docker-proxy::entrypoint" "Starting lazymc process for group: primary..."

    # wait for secondary lazymc process to start
    wait_for_formatted_log "lazymc-multi-server" "INFO" "lazymc-docker-proxy::entrypoint" "Starting lazymc process for group: secondary..."

    # wait for lazymc to start the primary proxy
    wait_for_formatted_log "lazymc-multi-server" "INFO" "primary::lazymc" "Proxying public 0.0.0.0:25565"

    # wait for lazymc to start the secondary proxy
    wait_for_formatted_log "lazymc-multi-server" "INFO" "secondary::lazymc" "Proxying public 0.0.0.0:25566"

    # wait for lazymc to start the primary server
    wait_for_formatted_log "lazymc-multi-server" "INFO" "primary::lazymc" "Starting server..."

    # wait for lazymc to start the secondary server
    wait_for_formatted_log "lazymc-multi-server" "INFO" "secondary::lazymc" "Starting server..."

    # wait for the primary server to be online
    wait_for_formatted_log "lazymc-multi-server" "INFO" "primary::lazymc::monitor" "Server is now online" 300

    #wait for the secondary server to be online
    wait_for_formatted_log "lazymc-multi-server" "INFO" "secondary::lazymc::monitor" "Server is now online" 300

    # wait for the primary server to be ready
    wait_for_log "primary" "RCON running on 0.0.0.0:25575"

    # wait for the secondary server to be ready
    wait_for_log "secondary" "RCON running on 0.0.0.0:25575"

    # wait for the primary server to be idle
    wait_for_formatted_log "lazymc-multi-server" "INFO" "primary::lazymc::monitor" "Server has been idle, sleeping..." 120

    # wait for the secondary server to be idle
    wait_for_formatted_log "lazymc-multi-server" "INFO" "secondary::lazymc::monitor" "Server has been idle, sleeping..." 120

    # wait for lazymc to put primary to sleep
    wait_for_formatted_log "lazymc-multi-server" "INFO" "primary::lazymc::monitor" "Server is now sleeping"

    # wait for lazymc to put secondary to sleep
    wait_for_formatted_log "lazymc-multi-server" "INFO" "secondary::lazymc::monitor" "Server is now sleeping"
}
