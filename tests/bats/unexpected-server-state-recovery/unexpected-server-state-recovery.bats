#!/usr/bin/env bats

load ../util.bash

project="./tests/bats/unexpected-server-state-recovery"

@test "Client connection test with static network" {
    # stop the client container
    stop_container mc-client-unexpected-server-state-recovery
    
    # stop the server container
    stop_container mc-unexpected-server-state-recovery

    # restart the lazymc container
    restart_container lazymc-unexpected-server-state-recovery

    # reset the start timestamp
    reset_timestamp

    #
    # The above steps ensures that the containers are in a clean state
    #

    # wait for lazymc-docker-proxy to to be ready
    wait_for_formatted_log "lazymc-unexpected-server-state-recovery" "INFO" "lazymc-docker-proxy::health" "Application is healthy."

    # wait for lazymc to be ready
    wait_for_formatted_log "lazymc-unexpected-server-state-recovery" "INFO" "mc::lazymc" "Proxying public 0.0.0.0:25565 to server 172.21.0.3:25565"

    # start the server container to put lazymc into an unexpected state
    start_container mc-unexpected-server-state-recovery

    # wait for the mincraft server to be ready
    wait_for_log "mc-unexpected-server-state-recovery" "RCON running on 0.0.0.0:25575" 300

    # start the client container to make lazymc want to shutdown the server
    start_container mc-client-unexpected-server-state-recovery

    # wait for test-bot to connect
    wait_for_log "mc-unexpected-server-state-recovery" "test-bot joined the game"

    # stop the client container
    stop_container mc-client-unexpected-server-state-recovery

    # wait for lazymc to be in an unexpected state
    wait_for_formatted_log "lazymc-unexpected-server-state-recovery" "WARN" "mc::lazymc" "Failed to stop server, no more suitable stopping method to use" 120

    # wait for lazymc-docker-proxy to detect the server is in an unexpected state
    wait_for_formatted_log "lazymc-unexpected-server-state-recovery" "WARN" "lazymc-docker-proxy::entrypoint" "Unexpected server state detected, force stopping mc server container..."

    # wait for the server to be stopped
    wait_for_formatted_log "lazymc-unexpected-server-state-recovery" "INFO" "lazymc-docker-proxy::entrypoint" "mc server container forcefully stopped"

    #
    # The above steps ensure that the app recovered, below steps ensure stable operation going forward
    #

    reset_timestamp

    # start the client container
    start_container mc-client-unexpected-server-state-recovery

    # wait for lazymc to start the server for the client
    wait_for_formatted_log "lazymc-unexpected-server-state-recovery" "INFO" "mc::lazymc" "Starting server for 'test-bot'..." 300

    # wait for the server to be online
    wait_for_formatted_log "lazymc-unexpected-server-state-recovery" "INFO" "mc::lazymc::monitor" "Server is now online" 300

    # wait for the mincraft server to be ready
    wait_for_log "mc-unexpected-server-state-recovery" "RCON running on 0.0.0.0:25575" 300

    # disconnect the client
    stop_container mc-client-unexpected-server-state-recovery

    # wait for the server to be idle
    wait_for_formatted_log "lazymc-unexpected-server-state-recovery" "INFO" "mc::lazymc::monitor" "Server has been idle, sleeping..." 120

    # wait for the server to be stopped
    wait_for_formatted_log "lazymc-unexpected-server-state-recovery" "INFO" "mc::lazymc-docker-proxy::command" "Received SIGTERM, stopping server..."

    # wait for the server to exit
    wait_for_log "mc-unexpected-server-state-recovery" "Thread RCON Listener stopped"

    # wait for lazymc to sleep
    wait_for_formatted_log "lazymc-unexpected-server-state-recovery" "INFO" "mc::lazymc::monitor" "Server is now sleeping"
}
