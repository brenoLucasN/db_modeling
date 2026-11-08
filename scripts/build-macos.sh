#!/usr/bin/env bash
# Build do binário release e geração de pacote macOS (.dmg e .app) com foco em chips ARM (Apple Silicon).
set -euo pipefail

cd "$(dirname "$0")/.."

NAME="db-modeling"
APP_NAME="DB Modeling"
VERSION="$(cargo pkgid -p app | cut -d# -f2 | cut -d: -f2 2>/dev/null || sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
DISTDIR="$(pwd)/dist"
BUILD_DIR="$(pwd)/target/macosbuild"
APP_DIR="$BUILD_DIR/$APP_NAME.app"
DMG_STAGE="$BUILD_DIR/dmg_stage"

# Seleção de arquitetura via argumento ($1), variável de ambiente (ARCH) ou detecção automática
REQ_ARCH="${1:-${ARCH:-arm64}}"

case "$REQ_ARCH" in
    arm|arm64|aarch64|apple-silicon|m1|m2|m3|m4)
        ARCH_NAME="arm64"
        RUST_TARGET="aarch64-apple-darwin"
        ;;
    intel|x86_64|amd64)
        ARCH_NAME="x86_64"
        RUST_TARGET="x86_64-apple-darwin"
        ;;
    universal|universal2)
        ARCH_NAME="universal"
        RUST_TARGET="universal"
        ;;
    *)
        ARCH_NAME="$REQ_ARCH"
        RUST_TARGET="aarch64-apple-darwin"
        ;;
esac

echo "=========================================================="
echo " DB Modeling - Build macOS (.dmg / .app)"
echo " Versão:     v$VERSION"
echo " Alvo:       Chips ARM (Apple Silicon / $ARCH_NAME)"
echo " Target:     $RUST_TARGET"
echo "=========================================================="

# Resolução ou compilação do executável
BIN_SRC=""

if [ -n "${TARGET_BIN:-}" ] && [ -f "$TARGET_BIN" ]; then
    echo "==> Utilizando binário pré-fornecido: $TARGET_BIN"
    BIN_SRC="$TARGET_BIN"
elif [[ "$(uname -s)" == "Darwin" ]]; then
    if [ "$RUST_TARGET" = "universal" ]; then
        echo "==> Compilando binário Universal (arm64 + x86_64)..."
        rustup target add aarch64-apple-darwin x86_64-apple-darwin 2>/dev/null || true
        cargo build --release -p app --target aarch64-apple-darwin
        cargo build --release -p app --target x86_64-apple-darwin
        mkdir -p "$BUILD_DIR/bin"
        lipo -create -output "$BUILD_DIR/bin/db-modeling" \
            target/aarch64-apple-darwin/release/app \
            target/x86_64-apple-darwin/release/app
        BIN_SRC="$BUILD_DIR/bin/db-modeling"
    elif [ "$(uname -m)" = "arm64" ] && [ "$ARCH_NAME" = "arm64" ]; then
        echo "==> Compilando nativamente em Apple Silicon (ARM64)..."
        cargo build --release -p app
        BIN_SRC="target/release/app"
    else
        echo "==> Compilando para target $RUST_TARGET..."
        rustup target add "$RUST_TARGET" 2>/dev/null || true
        cargo build --release -p app --target "$RUST_TARGET"
        BIN_SRC="target/$RUST_TARGET/release/app"
    fi
else
    # Host Linux / WSL / CI
    if [ -f "target/$RUST_TARGET/release/app" ]; then
        echo "==> Utilizando binário cross-compilado: target/$RUST_TARGET/release/app"
        BIN_SRC="target/$RUST_TARGET/release/app"
    elif [ -f "target/release/app" ]; then
        echo "==> Utilizando binário release local: target/release/app"
        BIN_SRC="target/release/app"
    else
        echo "==> Compilando DB Modeling em modo release..."
        cargo build --release -p app
        BIN_SRC="target/release/app"
    fi
fi

if [ ! -f "$BIN_SRC" ]; then
    echo "Erro: executável não encontrado em '$BIN_SRC'."
    exit 1
fi

echo "==> Preparando estrutura do bundle '$APP_NAME.app'..."
rm -rf "$BUILD_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"
mkdir -p "$DISTDIR"

# Instalação do binário
cp "$BIN_SRC" "$APP_DIR/Contents/MacOS/db-modeling"
chmod 0755 "$APP_DIR/Contents/MacOS/db-modeling"

# Geração do Info.plist com suporte a telas Retina e prioridade de execução ARM64
cat <<EOF > "$APP_DIR/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.dbmodeling.app</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>db-modeling</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>LSArchitecturePriority</key>
    <array>
        <string>arm64</string>
    </array>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>DBML Document</string>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>dbml</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Editor</string>
            <key>LSHandlerRank</key>
            <string>Owner</string>
        </dict>
    </array>
</dict>
</plist>
EOF
chmod 0644 "$APP_DIR/Contents/Info.plist"

# Geração do AppIcon.icns
echo "==> Gerando ícone Apple Icon Image (AppIcon.icns)..."
ICNS_OUT="$APP_DIR/Contents/Resources/AppIcon.icns"

if command -v iconutil &>/dev/null; then
    ICONSET_DIR="$BUILD_DIR/AppIcon.iconset"
    mkdir -p "$ICONSET_DIR"
    cp "crates/app/assets/brand/icon-16.png"   "$ICONSET_DIR/icon_16x16.png"
    cp "crates/app/assets/brand/icon-32.png"   "$ICONSET_DIR/icon_16x16@2x.png"
    cp "crates/app/assets/brand/icon-32.png"   "$ICONSET_DIR/icon_32x32.png"
    cp "crates/app/assets/brand/icon-64.png"   "$ICONSET_DIR/icon_32x32@2x.png"
    cp "crates/app/assets/brand/icon-128.png"  "$ICONSET_DIR/icon_128x128.png"
    cp "crates/app/assets/brand/icon-256.png"  "$ICONSET_DIR/icon_128x128@2x.png"
    cp "crates/app/assets/brand/icon-256.png"  "$ICONSET_DIR/icon_256x256.png"
    cp "crates/app/assets/brand/icon-1024.png" "$ICONSET_DIR/icon_512x512.png"
    cp "crates/app/assets/brand/icon-1024.png" "$ICONSET_DIR/icon_512x512@2x.png"
    iconutil -c icns "$ICONSET_DIR" -o "$ICNS_OUT"
elif command -v python3 &>/dev/null; then
    python3 - <<PYEOF
import struct, os

chunks = []
tags = [
    (b"ic07", "crates/app/assets/brand/icon-128.png"),
    (b"ic08", "crates/app/assets/brand/icon-256.png"),
    (b"ic10", "crates/app/assets/brand/icon-1024.png"),
]

for tag, path in tags:
    if os.path.exists(path):
        with open(path, "rb") as f:
            data = f.read()
        chunks.append((tag, data))

if chunks:
    total_len = 8 + sum(8 + len(d) for _, d in chunks)
    with open("$ICNS_OUT", "wb") as f:
        f.write(b"icns" + struct.pack(">I", total_len))
        for tag, d in chunks:
            f.write(tag + struct.pack(">I", 8 + len(d)) + d)
PYEOF
fi

if [ -f "$ICNS_OUT" ]; then
    chmod 0644 "$ICNS_OUT"
fi

# Adiciona script de desinstalação limpa dentro do pacote
cat <<'UNINST' > "$APP_DIR/Contents/Resources/uninstall.sh"
#!/bin/bash
# Desinstalador limpo do DB Modeling para macOS
echo "==> Desinstalando DB Modeling..."
rm -rf "/Applications/DB Modeling.app"
echo "==> DB Modeling desinstalado com sucesso do sistema."
UNINST
chmod 0755 "$APP_DIR/Contents/Resources/uninstall.sh"

# ============================================================
# Geração do Assistente de Instalação Apple Package (.pkg)
# ============================================================
PKG_OUT="$DISTDIR/${NAME}-v${VERSION}-${ARCH_NAME}.pkg"
if command -v pkgbuild &>/dev/null; then
    echo "==> Compilando Assistente de Instalação Apple Package (.pkg)..."
    pkgbuild --identifier "com.dbmodeling.app" \
             --version "${VERSION}" \
             --install-location "/Applications" \
             --component "$APP_DIR" \
             "$PKG_OUT"
    cp -f "$PKG_OUT" "$DISTDIR/${NAME}-macos-setup.pkg"
    echo " Assistente PKG gerado: $PKG_OUT"
fi

# Prepara diretório de staging do DMG
echo "==> Montando diretório de distribuição da imagem DMG..."
mkdir -p "$DMG_STAGE"
cp -R "$APP_DIR" "$DMG_STAGE/"
ln -sf /Applications "$DMG_STAGE/Applications"

DMG_OUT="$DISTDIR/${NAME}-v${VERSION}-${ARCH_NAME}.dmg"

echo "==> Gerando imagem de disco .dmg..."
if command -v hdiutil &>/dev/null; then
    rm -f "$DMG_OUT"
    hdiutil create -volname "${APP_NAME}" -srcfolder "$DMG_STAGE" -ov -format UDZO "$DMG_OUT"
elif command -v genisoimage &>/dev/null || command -v mkisofs &>/dev/null; then
    ISO_TOOL="$(command -v genisoimage || command -v mkisofs)"
    rm -f "$DMG_OUT"
    "$ISO_TOOL" -V "${APP_NAME}" -D -R -apple -no-pad -quiet -o "$DMG_OUT" "$DMG_STAGE"
else
    echo "Aviso: Nenhuma ferramenta de imagem de disco (hdiutil/genisoimage/mkisofs) foi encontrada."
    echo "Compactando bundle DB Modeling.app como arquivo .tar.gz..."
    tar -czf "$DISTDIR/${NAME}-v${VERSION}-${ARCH_NAME}.app.tar.gz" -C "$BUILD_DIR" "${APP_NAME}.app"
fi

# Se for arm64, cria também link/alias com aarch64 para máxima conveniência de download
if [ "$ARCH_NAME" = "arm64" ] && [ -f "$DMG_OUT" ]; then
    cp -f "$DMG_OUT" "$DISTDIR/${NAME}-v${VERSION}-aarch64.dmg"
fi

echo ""
echo "=========================================================="
echo " Pacote macOS para chips ARM (Apple Silicon) gerado com sucesso!"
echo " Localização do Bundle: $APP_DIR"
if [ -f "$DMG_OUT" ]; then
    echo " Imagem de disco DMG:   $DMG_OUT"
    ls -lh "$DMG_OUT"
    if [ -f "$DISTDIR/${NAME}-v${VERSION}-aarch64.dmg" ]; then
        echo " Alias para download:   $DISTDIR/${NAME}-v${VERSION}-aarch64.dmg"
    fi
fi
echo ""
echo " Para testar ou instalar no macOS com Apple Silicon (M1/M2/M3/M4):"
echo "   1. Abra o arquivo .dmg: open ./dist/${NAME}-v${VERSION}-${ARCH_NAME}.dmg"
echo "   2. Arraste 'DB Modeling.app' para a pasta 'Applications'."
echo "=========================================================="
