#!/usr/bin/env bash
# Build do binário release e geração de pacote Debian / Ubuntu (.deb).
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Compilando DB Modeling (App Desktop + CLI Web) em modo release..."
cargo build --release -p app -p dbml-cli

VERSION="$(cargo pkgid -p app | cut -d# -f2 | cut -d: -f2 2>/dev/null || sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
NAME="db-modeling"
BUILD_DIR="$(pwd)/target/debbuild"
DISTDIR="$(pwd)/dist"
PKG_DIR="$BUILD_DIR/pkg"

echo "==> Versão detectada no workspace: $VERSION"
echo "==> Preparando estrutura do pacote .deb..."

rm -rf "$BUILD_DIR"
mkdir -p "$PKG_DIR"/DEBIAN
mkdir -p "$PKG_DIR"/usr/bin
mkdir -p "$PKG_DIR"/usr/share/applications
mkdir -p "$DISTDIR"

# Binários (App Desktop + CLI Web) e atalho desktop
cp target/release/app "$PKG_DIR"/usr/bin/db-modeling
cp target/release/dbml "$PKG_DIR"/usr/bin/dbml
chmod 0755 "$PKG_DIR"/usr/bin/db-modeling "$PKG_DIR"/usr/bin/dbml

cat <<EOF > "$PKG_DIR/usr/share/applications/db-modeling.desktop"
[Desktop Entry]
Name=DB Modeling
GenericName=Modelador de Banco de Dados
Comment=Modele bancos de dados no canvas ou no texto com DBML
Exec=db-modeling %F
Icon=db-modeling
Terminal=false
Type=Application
Categories=Development;Database;IDE;
MimeType=text/x-dbml;
Keywords=dbml;database;diagram;modeling;canvas;schema;sql;
StartupWMClass=db-modeling
EOF
chmod 0644 "$PKG_DIR"/usr/share/applications/db-modeling.desktop

# Ícones hicolor
for size in 16 24 32 48 64 128 256; do
    dir="$PKG_DIR/usr/share/icons/hicolor/${size}x${size}/apps"
    mkdir -p "$dir"
    cp "crates/app/assets/brand/icon-${size}.png" "$dir/db-modeling.png"
    chmod 0644 "$dir/db-modeling.png"
done

dir_scalable="$PKG_DIR/usr/share/icons/hicolor/scalable/apps"
mkdir -p "$dir_scalable"
cp "crates/app/assets/brand/icon.svg" "$dir_scalable/db-modeling.svg"
chmod 0644 "$dir_scalable/db-modeling.svg"

# Arquivo DEBIAN/control
cat <<EOF > "$PKG_DIR/DEBIAN/control"
Package: ${NAME}
Version: ${VERSION}
Section: devel
Priority: optional
Architecture: amd64
Maintainer: DB Modeling Team <contato@dbmodeling.local>
Depends: libc6, libxkbcommon-x11-0, libxcb1, libfreetype6, libfontconfig1, libvulkan1, hicolor-icon-theme
Description: Modele bancos de dados no canvas ou no texto com DBML
 App desktop nativo para modelagem de banco de dados com foco total em UI/UX e
 edição bidirecional texto (DBML) e canvas visual, escrito em Rust e GPUI.
EOF
chmod 0644 "$PKG_DIR/DEBIAN/control"

DEB_OUT="$DISTDIR/${NAME}_${VERSION}_amd64.deb"

echo "==> Gerando arquivo .deb..."
if command -v dpkg-deb &>/dev/null; then
    dpkg-deb --build --root-owner-group "$PKG_DIR" "$DEB_OUT"
else
    # Fallback compatível usando ar + tar
    (cd "$PKG_DIR/DEBIAN" && tar --numeric-owner --group=0 --owner=0 -czf "$BUILD_DIR/control.tar.gz" control)
    (cd "$PKG_DIR" && tar --numeric-owner --group=0 --owner=0 -czf "$BUILD_DIR/data.tar.gz" usr)
    echo "2.0" > "$BUILD_DIR/debian-binary"
    ar -rc "$DEB_OUT" "$BUILD_DIR/debian-binary" "$BUILD_DIR/control.tar.gz" "$BUILD_DIR/data.tar.gz"
fi

echo ""
echo "=========================================================="
echo " Pacote Debian/Ubuntu (.deb) gerado com sucesso!"
echo " Localização:"
ls -lh "$DEB_OUT"
echo ""
echo " Para instalar ou atualizar no Ubuntu/Debian:"
echo "   sudo apt install ./dist/${NAME}_${VERSION}_amd64.deb"
echo " ou:"
echo "   sudo dpkg -i ./dist/${NAME}_${VERSION}_amd64.deb"
echo "=========================================================="
