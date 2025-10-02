#!/usr/bin/env bash

# Kubernetes-specific utilities for testing

CLUSTER_NAME="lazymc-test-cluster"
NAMESPACE="lazymc-test"

# Create a kind cluster
create_kind_cluster() {
    echo "Creating kind cluster: $CLUSTER_NAME" >&3
    kind create cluster --name $CLUSTER_NAME --config ./tests/bats/kubernetes/kind-config.yaml --wait 120s
}

# Delete the kind cluster
delete_kind_cluster() {
    echo "Deleting kind cluster: $CLUSTER_NAME" >&3
    kind delete cluster --name $CLUSTER_NAME
}

# Load docker image into kind cluster
load_image_to_kind() {
    local image=$1
    echo "Loading image $image into kind cluster" >&3
    kind load docker-image $image --name $CLUSTER_NAME
}

# Apply kubernetes manifests
apply_manifests() {
    echo "Applying Kubernetes manifests" >&3
    kubectl apply -f ./tests/bats/kubernetes/manifests.yaml
}

# Delete kubernetes manifests
delete_manifests() {
    echo "Deleting Kubernetes manifests" >&3
    kubectl delete -f ./tests/bats/kubernetes/manifests.yaml --ignore-not-found=true
}

# Wait for pod to be ready
wait_for_pod_ready() {
    local pod_name=$1
    local timeout=${2:-60}
    
    echo "Waiting for pod $pod_name to be ready" >&3
    kubectl wait --for=condition=ready pod -l app=$pod_name -n $NAMESPACE --timeout=${timeout}s
}

# Get pod logs
get_pod_logs() {
    local pod_name=$1
    local since=${2:-}
    
    if [ -n "$since" ]; then
        kubectl logs -l app=$pod_name -n $NAMESPACE --since-time=$since
    else
        kubectl logs -l app=$pod_name -n $NAMESPACE
    fi
}

# Wait for log line in pod
wait_for_pod_log() {
    local pod_name=$1
    local logline=$2
    local timeout=${3:-60}
    
    echo "Waiting for log in pod $pod_name: $logline" >&3
    
    trap 'exit 1' SIGINT SIGTERM
    until get_pod_logs $pod_name | grep -q "$logline";
    do
        if [ $timeout -eq 0 ]; then
            echo "Timeout waiting for log: $logline" >&3
            exit 1
        fi
        sleep 1
        ((timeout--))
    done
}

# Wait for formatted log line in pod
wait_for_pod_formatted_log() {
    local pod_name=$1
    local level=$2
    local target=$3
    local logline=$4
    local timeout=${5:-60}
    
    local regex="${level}\\s+${target}\\s+>\\s+${logline}"
    
    echo "Waiting for log in pod $pod_name: $level $target > $logline" >&3
    
    trap 'exit 1' SIGINT SIGTERM
    until get_pod_logs $pod_name | grep -qE "$regex";
    do
        if [ $timeout -eq 0 ]; then
            echo "Timeout waiting for log: $logline" >&3
            exit 1
        fi
        sleep 1
        ((timeout--))
    done
}

# Get deployment replica count
get_deployment_replicas() {
    local deployment_name=$1
    kubectl get deployment $deployment_name -n $NAMESPACE -o jsonpath='{.spec.replicas}'
}

# Scale deployment
scale_deployment() {
    local deployment_name=$1
    local replicas=$2
    echo "Scaling deployment $deployment_name to $replicas replicas" >&3
    kubectl scale deployment $deployment_name -n $NAMESPACE --replicas=$replicas
}

# Restart deployment
restart_deployment() {
    local deployment_name=$1
    echo "Restarting deployment: $deployment_name" >&3
    kubectl rollout restart deployment $deployment_name -n $NAMESPACE
    kubectl rollout status deployment $deployment_name -n $NAMESPACE --timeout=60s
}

# Delete pod
delete_pod() {
    local pod_selector=$1
    echo "Deleting pod with selector: $pod_selector" >&3
    kubectl delete pod -l $pod_selector -n $NAMESPACE --force --grace-period=0
}
