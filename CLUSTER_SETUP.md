# FESCA Cluster Setup - Schnellstart

## Was du brauchst:

### 1. Cluster-Zugang
- kubectl config vom Uni-Cluster
- Cluster-Ressourcen (CPU/Memory)

### 2. Docker Registry
- Registry-URL (z.B. `registry.uni.de`)
- Username/Password
- Push-Rechte

### 3. Image vorbereiten
```bash
# Build
docker build -t fesca:latest .

# Tag für Registry
docker tag fesca:latest registry.uni.de/fesca:latest

# Push
docker push registry.uni.de/fesca:latest
```

### 4. YAML-Dateien anpassen
In allen `*-deployment.yaml` und `client-job.yaml`:
```yaml
image: registry.uni.de/fesca:latest
imagePullPolicy: Always
```

### 5. Deployen
```bash
cd k8s
./deploy-all.sh
```

## Status-Check
```bash
kubectl get all -n fesca
kubectl logs -f job/fesca-client -n fesca
```

## Cleanup
```bash
kubectl delete namespace fesca
``` 