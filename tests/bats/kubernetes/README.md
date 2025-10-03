# Kubernetes Testing Guide

This directory contains integration tests for the Kubernetes backend of lazymc-docker-proxy.

## Prerequisites

- Docker
- kubectl
- kind (Kubernetes in Docker)
- BATS (Bash Automated Testing System)

## Running the Tests

The tests will automatically:
1. Build the lazymc-docker-proxy Docker image
2. Create a kind cluster
3. Load the image into the cluster
4. Deploy the manifests
5. Run the integration tests
6. Clean up the cluster

To run the tests:

```bash
bats tests/bats/kubernetes/kubernetes.bats
```

## Test Structure

- `kubernetes.bats` - Main test file with integration tests
- `k8s-util.bash` - Helper functions for Kubernetes operations
- `manifests.yaml` - Kubernetes manifests for the test setup
- `kind-config.yaml` - Kind cluster configuration

## What the Tests Verify

The tests verify that:
1. The lazymc-proxy correctly scales up a Deployment when a server needs to start
2. The Minecraft server pod starts successfully
3. The server becomes idle after the configured timeout
4. The lazymc-proxy correctly scales down the Deployment when the server is idle
5. The proxy correctly reads configuration from Deployment labels

## Troubleshooting

If tests fail:

1. Check that the kind cluster was created successfully:
   ```bash
   kind get clusters
   ```

2. Check the logs of the lazymc-proxy pod:
   ```bash
   kubectl logs -l app=lazymc-proxy -n lazymc-test
   ```

3. Check the Deployment status:
   ```bash
   kubectl get deployments -n lazymc-test
   ```

4. Delete the cluster and retry:
   ```bash
   kind delete cluster --name lazymc-test-cluster
   ```
