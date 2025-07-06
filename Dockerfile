# Verwende die offizielle Rust-Image als Basis
FROM rust:1.87 as builder

# Setze das Arbeitsverzeichnis
WORKDIR /app

# Kopiere die Cargo-Dateien zuerst (für besseres Caching)
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Kopiere die Proto-Datei
COPY proto/ ./proto/

# Kopiere den Quellcode
COPY src/ ./src/
COPY helpers/ ./helpers/
COPY data_owner/ ./data_owner/
COPY data_analyst/ ./data_analyst/
COPY computing_node/ ./computing_node/

# Installiere protobuf-compiler
RUN apt-get update && apt-get install -y protobuf-compiler

# Baue die Anwendung
RUN cargo build --release

# Zweite Stage: Runtime-Image
FROM debian:bookworm-slim

# Installiere notwendige Bibliotheken
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Erstelle einen nicht-root Benutzer
RUN useradd -m -u 1000 rustuser

# Setze das Arbeitsverzeichnis
WORKDIR /app

# Kopiere die kompilierte Anwendung
COPY --from=builder /app/target/release/fesca /app/fesca

# Kopiere die Konfigurationsdatei
COPY config.txt /app/config.txt

# Wechsle zum nicht-root Benutzer
USER rustuser

# Exponiere die Ports für gRPC
EXPOSE 50051 50052 50053

# Starte die Anwendung
CMD ["./fesca"] 