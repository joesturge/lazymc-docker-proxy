#!/usr/bin/env bats

load ../util.bash

project="./tests/bats/static-network"

@test "Client connection test with static network" {
    # stop the client container
    stop_container mc-client-static-network
    
    # stop the server container
    stop_container mc-static-network

    # restart the lazymc container
    restart_container lazymc-static-network

    # reset the start timestamp
    reset_timestamp

    #
    # The above steps ensures that the containers are in a clean state
    #

    # wait for lazymc to replace the address in the config
    wait_for_formatted_log "lazymc-static-network" "DEBUG" "lazymc-docker-proxy::docker" "Resolved address: 172.21.0.3:25565"

    # wait for the config to generated
    wait_for_log "lazymc-static-network" "address = \"172.21.0.3:25565\""

    # wait for lazymc-docker-proxy to to be ready
    wait_for_formatted_log "lazymc-static-network" "INFO" "lazymc-docker-proxy::health" "Application is healthy." 300

    # start the client container
    start_container mc-client-static-network

    # wait for lazymc to start the server for the client
    wait_for_formatted_log "lazymc-static-network" "INFO" "mc::lazymc" "Starting server for 'test-bot'..." 300

    # wait for the server to be online
    wait_for_formatted_log "lazymc-static-network" "INFO" "mc::lazymc::monitor" "Server is now online" 300

    # wait for the mincraft server to be ready
    wait_for_log "mc-static-network" "RCON running on 0.0.0.0:25575" 300

    # disconnect the client
    stop_container mc-client-static-network

    # wait for the server to be idle
    wait_for_formatted_log "lazymc-static-network" "INFO" "mc::lazymc::monitor" "Server has been idle, sleeping..." 120

    # wait for the server to be stopped
    wait_for_formatted_log "lazymc-static-network" "INFO" "mc::lazymc-docker-proxy::command" "Received SIGTERM, stopping server..."

    # wait for the server to exit
    wait_for_log "mc-static-network" "Thread RCON Listener stopped"

    # wait for lazymc to sleep
    wait_for_formatted_log "lazymc-static-network" "INFO" "mc::lazymc::monitor" "Server is now sleeping"
}
