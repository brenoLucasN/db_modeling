#!/usr/bin/env bash
# Sincroniza o núcleo Rust/Wasm, o Canvas Web, a CLI Web (assets embutidos),
# a Extensão VS Code e valida a compilação do App Desktop e do workspace.
#
# Uso:
#   ./scripts/sync-build.sh              # Sincroniza Wasm + Canvas Web + CLI assets + VS Code dist + cargo check
#   ./scripts/sync-build.sh --package    # Também gera o pacote .vsix da extensão VS Code
#   ./scripts/sync-build.sh --release    # Também compila os binários release (app e dbml-cli)

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

DO_PACKAGE=false
DO_RELEASE=false

for arg in "$@"; do
    case "$arg" in
        --package) DO_PACKAGE=true ;;
        --release) DO_RELEASE=true ;;
        *)
            echo "Uso: $0 [--package] [--release]"
            exit 1
            ;;
    esac
done

echo "==> [1/5] Compilando núcleo Rust para WebAssembly (dbml-wasm)..."
cargo build -p dbml-wasm --target wasm32-unknown-unknown --release

if ! command -v wasm-bindgen &>/dev/null; then
    echo "Erro: 'wasm-bindgen' não encontrado no PATH. Instale com: cargo install wasm-bindgen-cli"
    exit 1
fi

wasm-bindgen \
    --target web \
    --out-dir packages/canvas-web/src/wasm \
    target/wasm32-unknown-unknown/release/dbml_wasm.wasm

echo "==> [2/5] Compilando pacote compartilhado @dbmodeling/canvas-web (Lib + SPA)..."
(
    cd packages/canvas-web
    npm run build
    npm run build:app
)

echo "==> [3/5] Atualizando assets web embutidos na CLI (crates/cli/web/)..."
rm -rf crates/cli/web/*
mkdir -p crates/cli/web
cp -r packages/canvas-web/dist-app/* crates/cli/web/

echo "==> [4/5] Sincronizando e compilando Extensão VS Code (vscode-extension)..."
(
    cd vscode-extension
    npm run build
    if [ "$DO_PACKAGE" = true ]; then
        npm run package
    fi
)

echo "==> [5/5] Verificando integridade do workspace Rust (Desktop GPUI + CLI)..."
if [ "$DO_RELEASE" = true ]; then
    cargo build --release -p app -p dbml-cli
else
    cargo check --workspace
fi

echo "✓ Sincronização concluída! Desktop (app), CLI Web (dbml-cli) e VS Code estão alinhados."
