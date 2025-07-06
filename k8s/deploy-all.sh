#!/bin/bash

echo "🚀 Deploying FESCA to Kubernetes Cluster..."

# Create namespace
echo "📦 Creating namespace..."
kubectl apply -f namespace.yaml

# Create config
echo "⚙️  Creating config..."
kubectl apply -f configmap.yaml

# Deploy services
echo "🌐 Deploying services..."
kubectl apply -f services.yaml

# Deploy parties
echo "👥 Deploying parties..."
kubectl apply -f party1-deployment.yaml
kubectl apply -f party2-deployment.yaml
kubectl apply -f party3-deployment.yaml

# Wait for deployments to be ready
echo "⏳ Waiting for deployments to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/fesca-party1 -n fesca
kubectl wait --for=condition=available --timeout=300s deployment/fesca-party2 -n fesca
kubectl wait --for=condition=available --timeout=300s deployment/fesca-party3 -n fesca

# Run client job
echo "🎯 Running client job..."
kubectl apply -f client-job.yaml

echo "✅ Deployment complete!"
echo ""
echo "📊 Check status:"
echo "kubectl get all -n fesca"
echo ""
echo "📋 View logs:"
echo "kubectl logs -f job/fesca-client -n fesca"
echo ""
echo "🧹 Clean up:"
echo "kubectl delete namespace fesca" 