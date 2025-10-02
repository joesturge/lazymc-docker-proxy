#!/usr/bin/env bats

load k8s-util

setup_file() {
    # Build the docker image
    echo "Building lazymc-docker-proxy image..." >&3
    docker build -t lazymc-docker-proxy:test .
    
    # Create kind cluster
    create_kind_cluster
    
    # Load image into kind
    load_image_to_kind lazymc-docker-proxy:test
    
    # Apply manifests
    apply_manifests
    
    # Wait for lazymc-proxy to be ready
    sleep 10
    wait_for_pod_ready lazymc-proxy 120
}

teardown_file() {
    # Clean up
    delete_manifests
    delete_kind_cluster
}

@test "Kubernetes - Test lazymc stops server when idle" {
    # Restart the lazymc-proxy deployment to ensure clean state
    restart_deployment lazymc-proxy
    wait_for_pod_ready lazymc-proxy 120
    
    # Give it time to initialize
    sleep 5
    
    # Wait for lazymc process to start
    wait_for_pod_formatted_log "lazymc-proxy" "INFO" "lazymc-docker-proxy::entrypoint" "Starting lazymc process for group: mc..." 60
    
    # Wait for lazymc to start the server (it should scale up the deployment)
    wait_for_pod_formatted_log "lazymc-proxy" "INFO" "mc::lazymc" "Starting server..." 60
    
    # Check that the deployment was scaled up
    sleep 10
    replicas=$(get_deployment_replicas minecraft-server)
    [ "$replicas" -eq 1 ]
    
    # Wait for minecraft pod to be ready
    wait_for_pod_ready minecraft 300
    
    # Wait for the server to be online
    wait_for_pod_formatted_log "lazymc-proxy" "INFO" "mc::lazymc::monitor" "Server is now online" 300
    
    # Wait for the minecraft server to be ready
    wait_for_pod_log "minecraft" "RCON running on 0.0.0.0:25575" 300
    
    # Wait for the server to be idle
    wait_for_pod_formatted_log "lazymc-proxy" "INFO" "mc::lazymc::monitor" "Server has been idle, sleeping..." 120
    
    # Wait for the server to be stopped
    wait_for_pod_formatted_log "lazymc-proxy" "INFO" "mc::lazymc-docker-proxy::command" "Received SIGTERM, stopping server..." 60
    
    # Check that deployment was scaled down
    sleep 10
    replicas=$(get_deployment_replicas minecraft-server)
    [ "$replicas" -eq 0 ]
    
    # Wait for lazymc to sleep
    wait_for_pod_formatted_log "lazymc-proxy" "INFO" "mc::lazymc::monitor" "Server is now sleeping" 60
}
