# Docker Anleitung für FESCA

## Was ist Docker?
Docker ist ein Tool, um Anwendungen in "Containern" zu verpacken. Ein Container ist wie eine isolierte Umgebung, die alles enthält, was deine Anwendung braucht.

## Voraussetzungen
- Docker Desktop installiert (https://www.docker.com/products/docker-desktop/)
- Docker Compose (kommt mit Docker Desktop)

## Schnellstart

### 1. Docker Image bauen
```bash
# Im fesca-Verzeichnis ausführen
docker build -t fesca .
```

### 2. Mit Docker Compose starten (empfohlen)
```bash
# Alle drei Parteien und Client starten
docker-compose up --build

# Im Hintergrund starten
docker-compose up -d --build

# Logs anzeigen
docker-compose logs -f

# Stoppen
docker-compose down
```

### 3. Einzelne Container testen
```bash
# Nur Partei 1 starten
docker run -p 50051:50051 fesca

# Nur Partei 2 starten  
docker run -p 50052:50052 fesca

# Nur Partei 3 starten
docker run -p 50053:50053 fesca
```

## Was passiert?

1. **Docker baut deine Anwendung** in einem Container
2. **Drei separate Container** starten (P1, P2, P3)
3. **Jeder Container** läuft auf einem anderen Port:
   - P1: Port 50051
   - P2: Port 50052  
   - P3: Port 50053
4. **gRPC-Kommunikation** läuft zwischen den Containern

## Nützliche Befehle

```bash
# Container-Status anzeigen
docker ps

# In einen Container schauen
docker exec -it fesca-party1 /bin/bash

# Logs eines Containers anzeigen
docker logs fesca-party1

# Container stoppen
docker stop fesca-party1

# Alle Container löschen
docker-compose down --volumes --remove-orphans
```

## Für das Cluster

Diese Container können später auf euren Uni-Servern mit Kubernetes gestartet werden.

## Troubleshooting

### Port bereits belegt
```bash
# Prüfe, welche Ports belegt sind
lsof -i :50051
lsof -i :50052
lsof -i :50053
```

### Container startet nicht
```bash
# Logs anzeigen
docker logs <container-name>

# Container neu bauen
docker-compose down
docker-compose up --build
``` 