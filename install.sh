#!/usr/bin/env bash
# install.sh — Instala o Velha 2.0 como app desktop Linux
#
# Modos:
#   bash install.sh            → instala globalmente (requer senha de admin)
#   bash install.sh --local    → instala só para o usuário atual (sem sudo)
#
# Desinstalar:
#   bash install.sh --uninstall           (global)
#   bash install.sh --uninstall --local   (local)

set -e

PROJETO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARIO_NOME="velha2"
MODO_LOCAL=false
DESINSTALAR=false

# Parse de argumentos
for arg in "$@"; do
    case "$arg" in
        --local)      MODO_LOCAL=true ;;
        --uninstall)  DESINSTALAR=true ;;
    esac
done

# Destinos conforme o modo
if [ "$MODO_LOCAL" = true ]; then
    BINARIO_DESTINO="$HOME/.local/bin/${BINARIO_NOME}"
    ICONE_DESTINO="$HOME/.local/share/pixmaps/${BINARIO_NOME}.png"
    DESKTOP_DESTINO="$HOME/.local/share/applications/${BINARIO_NOME}.desktop"
    CMD_PRIVILEGIO=""
else
    BINARIO_DESTINO="/usr/local/bin/${BINARIO_NOME}"
    ICONE_DESTINO="/usr/share/pixmaps/${BINARIO_NOME}.png"
    DESKTOP_DESTINO="/usr/share/applications/${BINARIO_NOME}.desktop"
    CMD_PRIVILEGIO="pkexec"
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🎮 Velha 2.0 — Instalador Linux"
[ "$MODO_LOCAL" = true ] && echo "  (modo: usuário local)" || echo "  (modo: sistema global)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# ─── DESINSTALAR ────────────────────────────────────
if [ "$DESINSTALAR" = true ]; then
    echo "🗑️  Removendo arquivos instalados..."
    if [ "$MODO_LOCAL" = true ]; then
        rm -f "$BINARIO_DESTINO" "$ICONE_DESTINO" "$DESKTOP_DESTINO"
        update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
    else
        $CMD_PRIVILEGIO rm -f "$BINARIO_DESTINO" "$ICONE_DESTINO" "$DESKTOP_DESTINO"
        $CMD_PRIVILEGIO update-desktop-database 2>/dev/null || true
    fi
    echo "   ✅ Velha 2.0 desinstalado."
    exit 0
fi

# ─── INSTALAR ───────────────────────────────────────

# 1. Build release (só se o binário não existir ou for mais antigo que o código)
BINARIO_BUILD="${PROJETO_DIR}/target/release/${BINARIO_NOME}"
if [ ! -f "$BINARIO_BUILD" ] || [ "${PROJETO_DIR}/src/main.rs" -nt "$BINARIO_BUILD" ]; then
    echo "📦 Compilando binário release..."
    cd "${PROJETO_DIR}"
    cargo build --release --quiet
    echo "   ✅ Compilado: target/release/${BINARIO_NOME}"
else
    echo "📦 Binário release já está atualizado."
fi
echo ""

# 2. Instalar binário
echo "📂 Instalando binário em ${BINARIO_DESTINO}..."
if [ "$MODO_LOCAL" = true ]; then
    mkdir -p "$(dirname "$BINARIO_DESTINO")"
    install -Dm755 "${BINARIO_BUILD}" "${BINARIO_DESTINO}"
else
    $CMD_PRIVILEGIO install -Dm755 "${BINARIO_BUILD}" "${BINARIO_DESTINO}"
fi
echo "   ✅ Binário instalado"
echo ""

# 3. Instalar ícone
echo "🖼️  Instalando ícone..."
if [ "$MODO_LOCAL" = true ]; then
    mkdir -p "$(dirname "$ICONE_DESTINO")"
    install -Dm644 "${PROJETO_DIR}/assets/velha2.png" "${ICONE_DESTINO}"
    for TAMANHO in 48 64 128 256 512; do
        mkdir -p "$HOME/.local/share/icons/hicolor/${TAMANHO}x${TAMANHO}/apps"
        install -Dm644 "${PROJETO_DIR}/assets/velha2.png" \
            "$HOME/.local/share/icons/hicolor/${TAMANHO}x${TAMANHO}/apps/${BINARIO_NOME}.png"
    done
else
    $CMD_PRIVILEGIO install -Dm644 "${PROJETO_DIR}/assets/velha2.png" "${ICONE_DESTINO}"
    for TAMANHO in 48 64 128 256 512; do
        $CMD_PRIVILEGIO mkdir -p "/usr/share/icons/hicolor/${TAMANHO}x${TAMANHO}/apps"
        $CMD_PRIVILEGIO install -Dm644 "${PROJETO_DIR}/assets/velha2.png" \
            "/usr/share/icons/hicolor/${TAMANHO}x${TAMANHO}/apps/${BINARIO_NOME}.png"
    done
fi
echo "   ✅ Ícone instalado"
echo ""

# 4. Instalar .desktop (ajusta o Exec= para o destino correto)
echo "🖥️  Registrando entrada no menu de aplicativos..."
DESKTOP_TEMP="$(mktemp)"
sed "s|Exec=.*|Exec=${BINARIO_DESTINO}|" "${PROJETO_DIR}/velha2.desktop" > "$DESKTOP_TEMP"

if [ "$MODO_LOCAL" = true ]; then
    mkdir -p "$(dirname "$DESKTOP_DESTINO")"
    install -Dm644 "$DESKTOP_TEMP" "$DESKTOP_DESTINO"
    update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
else
    $CMD_PRIVILEGIO install -Dm644 "$DESKTOP_TEMP" "$DESKTOP_DESTINO"
    $CMD_PRIVILEGIO update-desktop-database 2>/dev/null || true
    $CMD_PRIVILEGIO gtk-update-icon-cache -f /usr/share/icons/hicolor 2>/dev/null || true
fi
rm -f "$DESKTOP_TEMP"
echo "   ✅ App registrado no menu"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  ✅ Instalação concluída!"
echo ""
echo "  Para rodar:"
echo "    Terminal:  velha2"
echo "    Menu:      Procure por 'Velha 2.0' nos aplicativos"
echo ""
echo "  Para desinstalar:"
[ "$MODO_LOCAL" = true ] \
    && echo "    bash install.sh --uninstall --local" \
    || echo "    bash install.sh --uninstall"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
