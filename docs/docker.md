# Como Rodar o Jogo da Velha 2.0 via Docker

Se você não quiser instalar as dependências de desenvolvimento do Rust no seu computador, você pode compilar e rodar o **Jogo da Velha 2.0** de forma isolada usando Docker.

Criamos um Dockerfile **multi-stage** e otimizado. Ele utiliza pouca memória durante a fase de build (compilação) e o ambiente final de execução (runtime) é super leve, carregando apenas os binários necessários para exibir a interface gráfica.

---

## 1. Como compilar (Build) a imagem

Estando na raiz do projeto (onde está o arquivo `Dockerfile`), execute o comando abaixo para gerar a imagem.

```bash
docker build -t jogodavelha2 .
```

*Nota: Durante o build, configuramos a variável `CARGO_BUILD_JOBS=1` para limitar as threads do compilador. Isso aumenta um pouco o tempo do processo, mas garante que o seu computador não engasgue e não tenha picos exagerados de consumo de memória RAM.*

---

## 2. Como Rodar o Jogo (Runtime)

Como o projeto é uma aplicação gráfica nativa (GUI), o contêiner precisa de permissão para desenhar janelas na interface visual (Display) da sua máquina (host). 

Para usuários de Linux rodando servidor gráfico X11, o passo a passo é:

**Passo A: Libere o acesso ao servidor X do seu host temporariamente**
```bash
xhost +local:docker
```

**Passo B: Execute o contêiner repassando o Display e Aceleração de Vídeo**
```bash
docker run -it --rm \
    --net=host \
    -e DISPLAY=$DISPLAY \
    -v /tmp/.X11-unix:/tmp/.X11-unix \
    --device /dev/dri \
    jogodavelha2
```

### O que esses parâmetros fazem?
- `--rm`: Deleta o contêiner quando o jogo for fechado.
- `--net=host`: Utiliza a rede do hospedeiro, caso vá usar o modo Multiplayer via P2P.
- `-e DISPLAY` e `-v /tmp/.X11-unix`: Compartilha a tela do host para a janela do jogo aparecer.
- `--device /dev/dri`: Permite que o contêiner use sua Placa de Vídeo (GPU) via aceleração por hardware (OpenGL), mantendo o jogo fluido.

**Passo C: Restrinja o acesso ao servidor X novamente (Opcional, por segurança)**
Ao terminar de jogar, você pode revogar a permissão do Docker rodando:
```bash
xhost -local:docker
```

## Usuários de Wayland
Se o seu Linux usa puramente Wayland (sem XWayland), você também pode precisar compartilhar o socket do Wayland:
```bash
docker run -it --rm \
    --net=host \
    -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY \
    -v $XDG_RUNTIME_DIR/$WAYLAND_DISPLAY:/tmp/$WAYLAND_DISPLAY \
    -e XDG_RUNTIME_DIR=/tmp \
    --device /dev/dri \
    jogodavelha2
```
