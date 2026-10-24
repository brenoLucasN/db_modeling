#!/usr/bin/env bash
# Build Windows + screenshot da janela do app em /tmp/dbm-shots/{dark,light}.png
set -euo pipefail
cd "$(dirname "$0")/.."
scripts/build-windows.sh
DESKTOP="${DBM_DESKTOP:-$(wslpath "$(powershell.exe -NoProfile -Command '[Environment]::GetFolderPath("Desktop")' | tr -d '\r')")}"
EXE="$(wslpath -w "$DESKTOP/db-modeling.exe")"
mkdir -p /tmp/dbm-shots
for m in dark light; do
  powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(wslpath -w scripts/screenshot.ps1)" \
    "$EXE" "$(wslpath -w /tmp/dbm-shots/$m.png)" $m
done
# edição ao vivo: editor aberto + digita uma tabela nova
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(wslpath -w scripts/screenshot.ps1)" \
  "$EXE" "$(wslpath -w /tmp/dbm-shots/editor.png)" dark edit
