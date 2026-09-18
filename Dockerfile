# Stage 1: Build (Ambiente de compilação)
FROM rust:1.80-slim AS builder

# Instala as dependências necessárias para compilar aplicações com egui/eframe no Linux
RUN apt-get update && apt-get install -y \
    pkg-config \
    libx11-dev \
    libasound2-dev \
    libudev-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Otimização: Cache das dependências do Cargo
# Copiamos apenas os arquivos do Cargo primeiro para baixar e compilar dependências
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Limita as threads de build para usar o mínimo de memória RAM possível do host
ENV CARGO_BUILD_JOBS=1

RUN cargo build --release
RUN rm -rf src

# Copia o resto do código
COPY . .
# Dá um "touch" para forçar a recompilação apenas do nosso código
RUN touch src/main.rs
RUN cargo build --release

# Stage 2: Runtime (Ambiente leve só para rodar)
FROM debian:bookworm-slim

# Instala apenas as bibliotecas de sistema necessárias em runtime (X11, Audio, OpenGL)
RUN apt-get update && apt-get install -y \
    libx11-6 \
    libasound2 \
    libudev1 \
    libxcb-render0 \
    libxcb-shape0 \
    libxcb-xfixes0 \
    libgl1-mesa-glx \
    libgl1-mesa-dri \
    && rm -rf /var/lib/apt/lists/*

# Cria um usuário não-root para rodar a aplicação gráfica de forma segura
RUN useradd -m appuser
USER appuser

WORKDIR /home/appuser/app

# Copia apenas o binário compilado do stage "builder" para não inchar a imagem final
COPY --from=builder /app/target/release/jogodavelha2 .
# Copia a pasta de assets (fontes, ícones)
COPY --from=builder /app/assets ./assets

CMD ["./jogodavelha2"]
