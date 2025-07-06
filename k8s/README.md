# FESCA Kubernetes Deployment

This directory contains Kubernetes manifests for deploying the FESCA correlated randomness protocol to a Kubernetes cluster.

## Prerequisites

1. **kubectl** installed and configured for your university cluster
2. **Docker image** pushed to a registry accessible by the cluster
3. **Kubernetes cluster** access (university cluster)

## Quick Start

### 1. Build and Push Docker Image

```bash
# Build image
cd fesca
docker build -t fesca:latest .

# Tag for your registry (replace with your registry)
docker tag fesca:latest your-registry.com/fesca:latest

# Push to registry
docker push your-registry.com/fesca:latest
```

### 2. Update Image References

Edit the deployment files to use your registry:
```yaml
# In party1-deployment.yaml, party2-deployment.yaml, party3-deployment.yaml, client-job.yaml
image: your-registry.com/fesca:latest
imagePullPolicy: Always  # Change from Never to Always
```

### 3. Deploy to Cluster

#### Option A: Deploy everything at once
```bash
kubectl apply -f k8s/
```

#### Option B: Deploy step by step
```bash
# Create namespace
kubectl apply -f k8s/namespace.yaml

# Create config
kubectl apply -f k8s/configmap.yaml

# Deploy services
kubectl apply -f k8s/services.yaml

# Deploy parties
kubectl apply -f k8s/party1-deployment.yaml
kubectl apply -f k8s/party2-deployment.yaml
kubectl apply -f k8s/party3-deployment.yaml

# Run client job
kubectl apply -f k8s/client-job.yaml
```

### 4. Monitor Deployment

```bash
# Check all resources
kubectl get all -n fesca

# Check pods status
kubectl get pods -n fesca

# View logs
kubectl logs -f deployment/fesca-party1 -n fesca
kubectl logs -f deployment/fesca-party2 -n fesca
kubectl logs -f deployment/fesca-party3 -n fesca
kubectl logs -f job/fesca-client -n fesca
```

### 5. Clean Up

```bash
kubectl delete namespace fesca
```

## Architecture

- **Namespace**: `fesca` - isolates all FESCA resources
- **ConfigMap**: `fesca-config` - application configuration
- **Deployments**: 3 parties running gRPC servers
- **Services**: Internal networking between parties
- **Job**: Client that coordinates the protocol

## Network Communication

- Party 1: `fesca-party1-service:50051`
- Party 2: `fesca-party2-service:50052`
- Party 3: `fesca-party3-service:50053`

## University Cluster Setup

### 1. Get Cluster Access
```bash
# Get kubeconfig from your university
kubectl config use-context your-cluster-context
```

### 2. Check Resources
```bash
# Check available resources
kubectl describe nodes
kubectl get nodes
```

### 3. Registry Setup
```bash
# Create image pull secret if needed
kubectl create secret docker-registry regcred \
  --docker-server=your-registry.com \
  --docker-username=your-username \
  --docker-password=your-password \
  --namespace=fesca
```

## Troubleshooting

### Image Pull Issues
```bash
# Check image pull events
kubectl describe pod <pod-name> -n fesca

# Check if image exists in registry
docker pull your-registry.com/fesca:latest
```

### Resource Issues
```bash
# Check resource usage
kubectl top pods -n fesca
kubectl describe nodes
```

### Network Issues
```bash
# Test service connectivity
kubectl run test-pod --image=busybox -n fesca --rm -it --restart=Never -- nslookup fesca-party1-service
```

### Check Events
```bash
kubectl get events -n fesca --sort-by='.lastTimestamp'
``` 