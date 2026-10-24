#!/usr/bin/env bash
# Cross-compila o app para Windows e copia o .exe para o Desktop do Windows.
set -euo pipefail
cd "$(dirname "$0")/.."
# Destino do .exe: Desktop do Windows (ou sobrescreva com DBM_DESKTOP=/mnt/c/...).
DESKTOP="${DBM_DESKTOP:-$(wslpath "$(powershell.exe -NoProfile -Command '[Environment]::GetFolderPath("Desktop")' | tr -d '\r')")}"
cargo xwin build -p app --target x86_64-pc-windows-msvc
# encerra só a instância de teste aberta, que trava a cópia do .exe
powershell.exe -NoProfile -Command "Stop-Process -Name db-modeling -Force -ErrorAction SilentlyContinue" || true
sleep 1
cp target/x86_64-pc-windows-msvc/debug/app.exe "$DESKTOP/db-modeling.exe"
# Build debug do GPUI compila o HLSL em runtime a partir do CARGO_MANIFEST_DIR (caminho Linux).
# No Windows esse caminho vira C:\home\...; espelhar os .hlsl lá.
src=$(echo ~/.cargo/registry/src/*/gpui-pre-windows-*/src)
dst=/mnt/c/${src#/}
mkdir -p "$dst" && cp "$src"/*.hlsl "$dst"/
