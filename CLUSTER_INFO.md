# Cluster Information - Bitte ausfüllen

## VPN Verbindung
- ✅ Tunnelblick mit .ovpn Datei
- ✅ SSH-Key in FusionDirectory

## Cluster Details (bitte ergänzen)
- **Cluster-URL**: _________________
- **kubectl config**: _________________
- **Namespace**: _________________

## Docker Registry (bitte ergänzen)
- **Registry-URL**: _________________
- **Username**: _________________
- **Password**: _________________

## Ressourcen
- **CPU**: _________________
- **Memory**: _________________
- **Storage**: _________________

## Zugang testen
```bash
# Nach VPN-Verbindung:
kubectl config get-contexts
kubectl get nodes
kubectl get namespaces
```

## Nächste Schritte
1. VPN verbinden
2. kubectl config laden
3. Cluster testen
4. Image pushen
5. Deployen 