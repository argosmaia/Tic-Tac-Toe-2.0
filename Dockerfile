# Stage 1: Build Linux Nativo
FROM rust:1.80-slim AS builder-linux

RUN apt-get update && apt-get install -y \
    pkg-config libx11-dev libasound2-dev libudev-dev \
    libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Build Windows (Cross-compilation)
FROM rust:1.80-slim AS builder-windows

RUN apt-get update && apt-get install -y \
    mingw-w64 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /app
COPY . .

# Compila para Windows 64-bits (Suporta Win 7, 8, 10, 11)
RUN cargo build --target x86_64-pc-windows-gnu --release

# Stage 3: Web Server para distribuir os executáveis
FROM nginx:alpine

# Limpa html padrão
RUN rm -rf /usr/share/nginx/html/*

# Cria pasta de downloads
RUN mkdir -p /usr/share/nginx/html/downloads

# Copia os binários construídos
COPY --from=builder-linux /app/target/release/jogodavelha2 /usr/share/nginx/html/downloads/jogodavelha2-linux
COPY --from=builder-windows /app/target/x86_64-pc-windows-gnu/release/jogodavelha2.exe /usr/share/nginx/html/downloads/jogodavelha2-windows.exe

# Cria uma página HTML simples para download
RUN echo '<!DOCTYPE html><html><head><meta charset="utf-8"><title>Download Jogo da Velha 2.0</title><style>body{font-family:sans-serif;text-align:center;margin-top:50px;background:#1e1e1e;color:#fff;} a{display:inline-block;margin:10px;padding:15px 25px;background:#4CAF50;color:#fff;text-decoration:none;border-radius:5px;font-weight:bold;} a:hover{background:#45a049;}</style></head><body><h1>Baixe o Jogo da Velha 2.0</h1><p>Os executáveis abaixo rodam nativamente, sem precisar do Docker no cliente.</p><div><a href="/downloads/jogodavelha2-windows.exe">📥 Download para Windows (7, 8, 10, 11)</a><a href="/downloads/jogodavelha2-linux">📥 Download para Linux</a></div><p style="margin-top:40px;color:#aaa;font-size:0.9em;">Nota sobre macOS: A compilação cruzada para Mac via Docker requer o SDK proprietário da Apple. Recomendamos usar o GitHub Actions para gerar o binário de Mac.</p><p style="color:#aaa;font-size:0.9em;">Nota sobre Versão Web (Wasm): O jogo usa "rusqlite" (SQLite C bindings) e "iroh" (Rede P2P), o que bloqueia a compilação direta para o navegador no momento sem refatorações no código-fonte.</p></body></html>' > /usr/share/nginx/html/index.html

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
