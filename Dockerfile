# =============================================================================
# PDP Facture - Dockerfile multi-stage
# =============================================================================
# Stage 1: Build Rust binary
# Stage 2: Runtime avec SaxonC-HE (natif), FOP, libxml2, qpdf
# =============================================================================

# --- Stage 1: Build ---
FROM rust:1.94-bookworm AS builder

# Dépendances de build pour libxml2 (binding Rust)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libxml2-dev \
    libxslt1-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copier les fichiers de dépendances d'abord (cache Docker)
COPY Cargo.toml Cargo.lock* ./
COPY crates/pdp-core/Cargo.toml crates/pdp-core/Cargo.toml
COPY crates/pdp-sftp/Cargo.toml crates/pdp-sftp/Cargo.toml
COPY crates/pdp-invoice/Cargo.toml crates/pdp-invoice/Cargo.toml
COPY crates/pdp-transform/Cargo.toml crates/pdp-transform/Cargo.toml
COPY crates/pdp-trace/Cargo.toml crates/pdp-trace/Cargo.toml
COPY crates/pdp-cdar/Cargo.toml crates/pdp-cdar/Cargo.toml
COPY crates/pdp-config/Cargo.toml crates/pdp-config/Cargo.toml
COPY crates/pdp-validate/Cargo.toml crates/pdp-validate/Cargo.toml
COPY crates/pdp-ereporting/Cargo.toml crates/pdp-ereporting/Cargo.toml
COPY crates/pdp-client/Cargo.toml crates/pdp-client/Cargo.toml
COPY crates/pdp-app/Cargo.toml crates/pdp-app/Cargo.toml

# Créer des src/lib.rs vides pour que cargo fetch fonctionne
RUN mkdir -p crates/pdp-core/src && echo "pub fn _dummy() {}" > crates/pdp-core/src/lib.rs && \
    mkdir -p crates/pdp-sftp/src && echo "pub fn _dummy() {}" > crates/pdp-sftp/src/lib.rs && \
    mkdir -p crates/pdp-invoice/src && echo "pub fn _dummy() {}" > crates/pdp-invoice/src/lib.rs && \
    mkdir -p crates/pdp-transform/src && echo "pub fn _dummy() {}" > crates/pdp-transform/src/lib.rs && \
    mkdir -p crates/pdp-trace/src && echo "pub fn _dummy() {}" > crates/pdp-trace/src/lib.rs && \
    mkdir -p crates/pdp-cdar/src && echo "pub fn _dummy() {}" > crates/pdp-cdar/src/lib.rs && \
    mkdir -p crates/pdp-config/src && echo "pub fn _dummy() {}" > crates/pdp-config/src/lib.rs && \
    mkdir -p crates/pdp-validate/src && echo "pub fn _dummy() {}" > crates/pdp-validate/src/lib.rs && \
    mkdir -p crates/pdp-ereporting/src && echo "pub fn _dummy() {}" > crates/pdp-ereporting/src/lib.rs && \
    mkdir -p crates/pdp-client/src && echo "pub fn _dummy() {}" > crates/pdp-client/src/lib.rs && \
    mkdir -p crates/pdp-app/src && echo "fn main() {}" > crates/pdp-app/src/main.rs

# Pré-télécharger les dépendances
RUN cargo fetch 2>/dev/null || true

# Copier le code source complet
COPY crates/ crates/
COPY tests/ tests/
COPY specs/ specs/
COPY config.yaml .

# Build release
RUN cargo build --release --bin pdp

# Build tests (pour pouvoir les exécuter dans le conteneur)
RUN cargo test --release --no-run 2>/dev/null || true

# --- Stage 2: Runtime ---
FROM debian:bookworm-slim AS runtime

# Installer les dépendances runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
    # libxml2 pour la validation XSD (runtime)
    libxml2 \
    libxslt1.1 \
    xsltproc \
    # Java uniquement pour Apache FOP (rendu PDF)
    default-jre-headless \
    # qpdf pour la correction header binaire PDF/A-3
    qpdf \
    # Utilitaires
    ca-certificates \
    curl \
    unzip \
    # gcc nécessaire pour compiler le binaire SaxonC transform
    gcc \
    libc6-dev \
    && rm -rf /var/lib/apt/lists/*

# Installer SaxonC-HE (natif C++, pas de JVM pour les transformations XSLT)
ARG SAXONC_VERSION=12-9-0
RUN ARCH=$(dpkg --print-architecture) && \
    case "$ARCH" in \
        amd64) SAXONC_ARCH="linux-x86_64" ;; \
        arm64) SAXONC_ARCH="linux-arm64" ;; \
        *) echo "Architecture non supportée: $ARCH" && exit 1 ;; \
    esac && \
    curl -fsSL "https://downloads.saxonica.com/SaxonC/HE/12/SaxonCHE-${SAXONC_ARCH}-${SAXONC_VERSION}.zip" \
    -o /tmp/saxonc.zip && \
    mkdir -p /opt/saxonc && \
    unzip -q /tmp/saxonc.zip -d /opt/saxonc && \
    rm /tmp/saxonc.zip && \
    # Compiler le binaire CLI 'transform'
    cd /opt/saxonc/command && \
    ./build64-linux.sh && \
    cp /opt/saxonc/command/transform /usr/local/bin/transform && \
    chmod +x /usr/local/bin/transform && \
    # Rendre la lib partagée accessible
    echo "/opt/saxonc/libs" > /etc/ld.so.conf.d/saxonc.conf && ldconfig && \
    # Nettoyer gcc après compilation (plus nécessaire au runtime)
    apt-get update && apt-get remove -y gcc libc6-dev unzip && apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

# Installer Apache FOP (génération PDF Factur-X)
ARG FOP_VERSION=2.11
RUN curl -fsSL "https://dlcdn.apache.org/xmlgraphics/fop/binaries/fop-${FOP_VERSION}-bin.tar.gz" \
    -o /tmp/fop.tar.gz && \
    tar xzf /tmp/fop.tar.gz -C /opt && \
    ln -s /opt/fop-${FOP_VERSION}/fop/fop /usr/local/bin/fop && \
    rm /tmp/fop.tar.gz

# Créer l'utilisateur non-root
RUN useradd -m -s /bin/bash pdp

WORKDIR /app

# Copier le binaire depuis le builder
COPY --from=builder /app/target/release/pdp /usr/local/bin/pdp

# Copier les specs (XSD, Schematron, XSLT)
COPY --from=builder /app/specs/ /app/specs/

# Copier la config par défaut
COPY --from=builder /app/config.yaml /app/config.yaml

# Créer les répertoires de données
RUN mkdir -p /app/data/in/ubl /app/data/in/cii /app/data/out/processed /app/data/out/errors /app/data/out/cdar && \
    chown -R pdp:pdp /app

USER pdp

# Variables d'environnement
ENV PDP_SPECS_DIR=/app/specs
ENV PDP_CONFIG=/app/config.yaml
ENV ELASTICSEARCH_URL=http://elasticsearch:9200
ENV RUST_LOG=info

# Healthcheck
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD pdp stats --config $PDP_CONFIG 2>/dev/null || exit 1

ENTRYPOINT ["pdp"]
CMD ["start"]
