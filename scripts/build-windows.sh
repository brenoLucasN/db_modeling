#!/usr/bin/env bash
# Cross-compila o app para Windows (.exe) e exporta para dist/ e Desktop do Windows.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION="$(cargo pkgid -p app | cut -d# -f2 | cut -d: -f2 2>/dev/null || sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
NAME="db-modeling"
DISTDIR="$(pwd)/dist"
mkdir -p "$DISTDIR"

MODE="${1:-release}"
if [ "$MODE" = "debug" ]; then
    echo "==> Compilando DB Modeling v$VERSION para Windows (App + CLI em modo debug)..."
    cargo xwin build -p app -p dbml-cli --target x86_64-pc-windows-msvc
    EXE_SRC="target/x86_64-pc-windows-msvc/debug/app.exe"
    CLI_SRC="target/x86_64-pc-windows-msvc/debug/dbml.exe"
else
    echo "==> Compilando DB Modeling v$VERSION para Windows (App + CLI em modo release)..."
    cargo xwin build --release -p app -p dbml-cli --target x86_64-pc-windows-msvc
    EXE_SRC="target/x86_64-pc-windows-msvc/release/app.exe"
    CLI_SRC="target/x86_64-pc-windows-msvc/release/dbml.exe"
fi

EXE_DIST="$DISTDIR/${NAME}-v${VERSION}-x86_64.exe"
CLI_DIST="$DISTDIR/dbml-v${VERSION}-x86_64.exe"
cp "$EXE_SRC" "$EXE_DIST"
if [ -f "$CLI_SRC" ]; then
    cp "$CLI_SRC" "$CLI_DIST"
fi

echo ""
echo "=========================================================="
echo " Executável Windows (.exe) gerado com sucesso!"
echo " Localização: $EXE_DIST"

# Se executado no WSL2 com Desktop Windows acessível
if command -v powershell.exe &>/dev/null; then
    DESKTOP="${DBM_DESKTOP:-$(wslpath "$(powershell.exe -NoProfile -Command '[Environment]::GetFolderPath("Desktop")' 2>/dev/null | tr -d '\r')")}"
    if [ -d "$DESKTOP" ]; then
        powershell.exe -NoProfile -Command "Stop-Process -Name db-modeling -Force -ErrorAction SilentlyContinue" 2>/dev/null || true
        sleep 1
        cp "$EXE_DIST" "$DESKTOP/db-modeling.exe"
        echo " Copiado para o Desktop do Windows: $DESKTOP/db-modeling.exe"
    fi
fi
echo "=========================================================="
