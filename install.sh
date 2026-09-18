#!/usr/bin/env bash

# install.sh — Instala o Jogo da Velha 2.0 no Ubuntu/Linux

#

# Instalação global:

# bash install.sh

#

# Instalação local:

# bash install.sh --local

#

# Desinstalação global:

# bash install.sh --uninstall

#

# Desinstalação local:

# bash install.sh --uninstall --local

set -e

PROJETO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

BINARIO_NOME="jogodavelha2.0"
ICONE_NOME="tictactoe.png"
DESKTOP_NOME="jogodavelha2.0.desktop"

MODO_LOCAL=false
DESINSTALAR=false

for arg in "$@"; do
case "$arg" in
--local)
MODO_LOCAL=true
;;
--uninstall)
DESINSTALAR=true
;;
*)
echo "Argumento desconhecido: $arg"
exit 1
;;
esac
done

if [ "$MODO_LOCAL" = true ]; then
BINARIO_DESTINO="$HOME/.local/bin/$BINARIO_NOME"
ICONE_DESTINO="$HOME/.local/share/icons/hicolor/256x256/apps/$ICONE_NOME"
DESKTOP_DESTINO="$HOME/.local/share/applications/$DESKTOP_NOME"
else
BINARIO_DESTINO="/usr/local/bin/$BINARIO_NOME"
ICONE_DESTINO="/usr/share/icons/hicolor/256x256/apps/$ICONE_NOME"
DESKTOP_DESTINO="/usr/share/applications/$DESKTOP_NOME"
fi

echo "=============================================="
echo "  Jogo da Velha 2.0 - Instalador Linux"
echo "=============================================="
echo ""

# --------------------------------------------------

# DESINSTALAÇÃO

# --------------------------------------------------

if [ "$DESINSTALAR" = true ]; then

```
echo "Removendo instalação..."

if [ "$MODO_LOCAL" = true ]; then
    rm -f "$BINARIO_DESTINO"
    rm -f "$ICONE_DESTINO"
    rm -f "$DESKTOP_DESTINO"

    update-desktop-database \
        "$HOME/.local/share/applications" \
        2>/dev/null || true
else
    sudo rm -f "$BINARIO_DESTINO"
    sudo rm -f "$ICONE_DESTINO"
    sudo rm -f "$DESKTOP_DESTINO"

    sudo update-desktop-database \
        2>/dev/null || true

    sudo gtk-update-icon-cache \
        -f /usr/share/icons/hicolor \
        2>/dev/null || true
fi

echo "Instalação removida."
exit 0
```

fi

# --------------------------------------------------

# VALIDAÇÃO

# --------------------------------------------------

BINARIO_ORIGEM="$PROJETO_DIR/target/release/jogodavelha2"
ICONE_ORIGEM="$PROJETO_DIR/assets/tictactoe.png"

if [ ! -f "$BINARIO_ORIGEM" ]; then
echo "Erro: binário não encontrado:"
echo "  $BINARIO_ORIGEM"
exit 1
fi

if [ ! -f "$ICONE_ORIGEM" ]; then
echo "Erro: ícone não encontrado:"
echo "  $ICONE_ORIGEM"
exit 1
fi

chmod +x "$BINARIO_ORIGEM"

# --------------------------------------------------

# INSTALAÇÃO

# --------------------------------------------------

echo "Instalando binário..."

if [ "$MODO_LOCAL" = true ]; then

```
install -Dm755 \
    "$BINARIO_ORIGEM" \
    "$BINARIO_DESTINO"
```

else

```
sudo install -Dm755 \
    "$BINARIO_ORIGEM" \
    "$BINARIO_DESTINO"
```

fi

echo "Binário instalado em:"
echo "  $BINARIO_DESTINO"
echo ""

# --------------------------------------------------

# ÍCONE

# --------------------------------------------------

echo "Instalando ícone..."

if [ "$MODO_LOCAL" = true ]; then

```
install -Dm644 \
    "$ICONE_ORIGEM" \
    "$ICONE_DESTINO"
```

else

```
sudo install -Dm644 \
    "$ICONE_ORIGEM" \
    "$ICONE_DESTINO"
```

fi

echo "Ícone instalado."
echo ""

# --------------------------------------------------

# DESKTOP

# --------------------------------------------------

echo "Registrando aplicativo..."

DESKTOP_TEMP="$(mktemp)"
sed "s|^Exec=.*|Exec=$BINARIO_DESTINO|" \
    "$PROJETO_DIR/$DESKTOP_NOME" \
    | sed "s|^Icon=.*|Icon=$ICONE_DESTINO|" > "$DESKTOP_TEMP"

if [ "$MODO_LOCAL" = true ]; then

```
install -Dm644 \
    "$DESKTOP_TEMP" \
    "$DESKTOP_DESTINO"

update-desktop-database \
    "$HOME/.local/share/applications" \
    2>/dev/null || true
```

else

```
sudo install -Dm644 \
    "$DESKTOP_TEMP" \
    "$DESKTOP_DESTINO"

sudo update-desktop-database \
    2>/dev/null || true

sudo gtk-update-icon-cache \
    -f /usr/share/icons/hicolor \
    2>/dev/null || true
```

fi

rm -f "$DESKTOP_TEMP"

echo "Aplicativo registrado."
echo ""

echo "=============================================="
echo "  Instalação concluída!"
echo "=============================================="
echo ""
echo "Executar pelo terminal:"
echo "  $BINARIO_NOME"
echo ""
echo "Ou procure por:"
echo "  Jogo da Velha 2.0"
echo ""
echo "Desinstalar:"
if [ "$MODO_LOCAL" = true ]; then
echo "  bash install.sh --uninstall --local"
else
echo "  bash install.sh --uninstall"
fi
