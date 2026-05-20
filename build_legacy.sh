#!/bin/bash
set -e

echo "🐳 Iniciando build de compatibilidade no Docker (Ubuntu 20.04 / GLIBC 2.31)..."

# Criando um Dockerfile temporário para evitar mexer nas permissões do seu host
cat << 'EOF' > Dockerfile.legacy
FROM ubuntu:20.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
    curl build-essential libxcb-render0-dev \
    libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libssl-dev pkg-config \
    libfontconfig1-dev tar

# Instala o Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

# Compila o release usando as bibliotecas antigas do Ubuntu 20.04
RUN cargo build --release

# Organiza a arvore do pacote
RUN mkdir -p velha2-linux-v0.1.1-legacy/assets \
    && mkdir -p velha2-linux-v0.1.1-legacy/target/release \
    && cp target/release/velha2 velha2-linux-v0.1.1-legacy/target/release/ \
    && cp assets/velha2.png velha2-linux-v0.1.1-legacy/assets/ \
    && cp install.sh velha2-linux-v0.1.1-legacy/ \
    && cp velha2.desktop velha2-linux-v0.1.1-legacy/ \
    && cp README.md velha2-linux-v0.1.1-legacy/ \
    && tar -czvf velha2-linux-v0.1.1-legacy.tar.gz velha2-linux-v0.1.1-legacy
EOF

echo "⏳ Compilando imagem e jogo no container (isso pode levar uns 3-5 minutos)..."
docker build -t velha2-legacy-builder -f Dockerfile.legacy .

echo "📦 Extraindo o pacote .tar.gz novo gerado..."
# Puxa o arquivo de dentro da imagem recém-construída para sua máquina
docker run --rm --name extractor -d velha2-legacy-builder tail -f /dev/null
docker cp extractor:/app/velha2-linux-v0.1.1-legacy.tar.gz ./
docker stop extractor

# Limpa o lixo
rm Dockerfile.legacy
docker rmi velha2-legacy-builder

echo "✅ Feito! O arquivo velha2-linux-v0.1.1-legacy.tar.gz foi gerado na raiz do seu projeto!"
