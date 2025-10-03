# Kubernetes Support Implementation Summary

This document summarizes the Kubernetes support implementation for lazymc-docker-proxy.

## Overview

The lazymc-docker-proxy now supports running on Kubernetes clusters in addition to Docker environments. The implementation uses the kube-rs library to interact with the Kubernetes API.

## Architecture

### Backend Abstraction Layer

A new `backend` module provides an abstraction layer that allows the application to work with both Docker and Kubernetes:

- **Backend Trait**: Defines common operations (start, stop, stop_all, get_labels)
- **DockerBackend**: Implementation for Docker (wraps existing docker.rs functions)
- **KubernetesBackend**: Implementation for Kubernetes (wraps new kubernetes.rs functions)

### Backend Selection

The backend is selected automatically based on the `LAZYMC_BACKEND` environment variable:
- `LAZYMC_BACKEND=kubernetes` or `LAZYMC_BACKEND=k8s` → Kubernetes backend
- Default (no env var or any other value) → Docker backend (backward compatible)

## Implementation Details

### New Files

1. **src/backend.rs** (83 lines)
   - Backend trait and implementations
   - Automatic backend selection logic

2. **src/kubernetes.rs** (295 lines)
   - Kubernetes API client connection
   - Pod/Deployment/StatefulSet management functions
   - Label retrieval from resources
   - Scaling operations

3. **tests/bats/kubernetes/** (5 files)
   - Complete integration test suite using kind
   - Helper utilities for Kubernetes operations
   - Test manifests and configuration

### Modified Files

1. **src/main.rs**
   - Added backend and kubernetes module imports

2. **src/entrypoint/mod.rs**
   - Replaced direct docker calls with backend abstraction
   - Uses `backend::get_backend()` to get appropriate backend

3. **src/command/mod.rs**
   - Updated to use backend abstraction for start/stop operations

4. **Cargo.toml**
   - Added kube-rs dependencies: kube (v0.96.0), k8s-openapi (v0.23.0)
   - Added serde_json for JSON operations

5. **README.md**
   - Added comprehensive Kubernetes deployment guide
   - Documented RBAC requirements
   - Provided complete example manifests

## How It Works

### Kubernetes Mode Operation

1. **Initialization**:
   - Proxy pod connects to Kubernetes API using in-cluster credentials
   - Scans for Deployments/StatefulSets with `lazymc.enabled=true` label
   - Reads configuration from pod template labels

2. **When Player Connects**:
   - lazymc detects player connection
   - Calls backend.start(group)
   - Kubernetes backend patches Deployment/StatefulSet to scale replicas to 1
   - Waits for pod to become ready
   - Connects player to the Minecraft server

3. **When Server Idle**:
   - lazymc detects server is idle
   - Calls backend.stop(group)
   - Kubernetes backend patches Deployment/StatefulSet to scale replicas to 0
   - Pod terminates gracefully

### Label Configuration

The same label format works for both Docker and Kubernetes:

```yaml
labels:
  lazymc.enabled: "true"
  lazymc.group: "mc"
  lazymc.server.address: "minecraft-server:25565"
  lazymc.time.minimum_online_time: "30"
  lazymc.time.sleep_after: "60"
  # ... all other lazymc configuration options
```

For Kubernetes, apply labels to:
- Deployment/StatefulSet metadata
- Pod template metadata

## RBAC Requirements

The proxy needs specific permissions in Kubernetes:

```yaml
rules:
- apiGroups: [""]
  resources: ["pods"]
  verbs: ["get", "list", "watch", "delete"]
- apiGroups: ["apps"]
  resources: ["deployments", "statefulsets"]
  verbs: ["get", "list", "watch", "patch"]
```

## Testing

### Integration Tests

Complete test suite using kind (Kubernetes in Docker):
- Automatically creates/destroys test cluster
- Verifies scaling up when server starts
- Verifies scaling down when server idle
- Tests label-based configuration

Run tests with:
```bash
bats tests/bats/kubernetes/kubernetes.bats
```

### Test Coverage

- Backend selection logic
- Kubernetes API connectivity
- Deployment scaling operations
- Label retrieval from Deployments (when no pods exist)
- Label retrieval from running pods
- Complete start/idle/stop cycle

## Key Differences from Docker Mode

| Aspect | Docker Mode | Kubernetes Mode |
|--------|-------------|-----------------|
| **Resource Type** | Containers | Pods (via Deployments/StatefulSets) |
| **Start/Stop** | docker start/stop | Scale replicas 0↔1 |
| **Discovery** | Docker socket | Kubernetes API |
| **Networking** | Docker networks | Kubernetes Services |
| **IP Assignment** | Static IP via IPAM | Service DNS |
| **Permissions** | Docker socket access | RBAC Role |

## Backward Compatibility

- Existing Docker configurations continue to work unchanged
- Default behavior (no LAZYMC_BACKEND env var) is Docker mode
- No breaking changes to existing functionality
- All lazymc configuration options supported in both modes

## Dependencies Added

- `kube = "0.96.0"` - Kubernetes client library
- `k8s-openapi = "0.23.0"` - Kubernetes API types
- `serde_json = "1.0"` - JSON serialization for API operations

## Future Enhancements

Potential improvements for future versions:
- Support for DaemonSets
- Cross-namespace management
- Helm chart for easier deployment
- Metrics and monitoring integration
- Support for custom resource definitions (CRDs)
