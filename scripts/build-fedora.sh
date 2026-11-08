#!/usr/bin/env bash
# Build do binário release e geração de pacote RPM para Fedora / DNF.
set -euo pipefail

cd "$(dirname "$0")/.."

echo "==> Verificando dependências de empacotamento..."
if ! command -v rpmbuild &>/dev/null; then
    echo "Erro: 'rpmbuild' não foi encontrado."
    echo "Instale com: sudo dnf install rpm-build"
    exit 1
fi

echo "==> Compilando DB Modeling (App Desktop + CLI Web) em modo release..."
cargo build --release -p app -p dbml-cli

VERSION="$(cargo pkgid -p app | cut -d# -f2 | cut -d: -f2 2>/dev/null || sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
NAME="db-modeling"
TOPDIR="$(pwd)/target/rpmbuild"
DISTDIR="$(pwd)/dist"

echo "==> Versão detectada no workspace: $VERSION"
echo "==> Preparando estrutura do RPM..."
rm -rf "$TOPDIR"
mkdir -p "$TOPDIR"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}
mkdir -p "$DISTDIR"

STAGE_DIR="$TOPDIR/SOURCES/${NAME}-${VERSION}"
mkdir -p "$STAGE_DIR/icons"

cp target/release/app "$STAGE_DIR/db-modeling"
cp target/release/dbml "$STAGE_DIR/dbml"
cp crates/app/assets/brand/icon-*.png "$STAGE_DIR/icons/"
cp crates/app/assets/brand/icon.svg "$STAGE_DIR/icons/"

# Gera db-modeling.desktop inline
cat <<EOF > "$STAGE_DIR/db-modeling.desktop"
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

tar -czf "$TOPDIR/SOURCES/${NAME}-${VERSION}.tar.gz" -C "$TOPDIR/SOURCES" "${NAME}-${VERSION}"
rm -rf "$STAGE_DIR"

# Gera o db-modeling.spec inline parametrizado
cat <<EOF > "$TOPDIR/SPECS/db-modeling.spec"
%global debug_package %{nil}

Name:           ${NAME}
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        Modele bancos de dados no canvas ou no texto com DBML

License:        Apache-2.0
URL:            https://github.com/smovisk/db_modeling
Source0:        %{name}-%{version}.tar.gz

BuildArch:      x86_64

Requires:       libxkbcommon-x11%{?_isa}
Requires:       libxcb%{?_isa}
Requires:       freetype%{?_isa}
Requires:       fontconfig%{?_isa}
Requires:       vulkan-loader%{?_isa}
Requires:       hicolor-icon-theme

%description
App desktop nativo para modelagem de banco de dados com foco total em UI/UX e
edição bidirecional texto (DBML) e canvas visual, escrito em Rust e GPUI.

%prep
%autosetup

%build
# O binário release pré-compilado é empacotado

%install
rm -rf %{buildroot}
install -d -m 0755 %{buildroot}%{_bindir}
install -m 0755 db-modeling %{buildroot}%{_bindir}/db-modeling
install -m 0755 dbml %{buildroot}%{_bindir}/dbml

install -d -m 0755 %{buildroot}%{_datadir}/applications
install -m 0644 db-modeling.desktop %{buildroot}%{_datadir}/applications/db-modeling.desktop

for size in 16 24 32 48 64 128 256; do
    install -d -m 0755 %{buildroot}%{_datadir}/icons/hicolor/\${size}x\${size}/apps
    install -m 0644 icons/icon-\${size}.png %{buildroot}%{_datadir}/icons/hicolor/\${size}x\${size}/apps/db-modeling.png
done

install -d -m 0755 %{buildroot}%{_datadir}/icons/hicolor/scalable/apps
install -m 0644 icons/icon.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/db-modeling.svg

%files
%{_bindir}/db-modeling
%{_bindir}/dbml
%{_datadir}/applications/db-modeling.desktop
%{_datadir}/icons/hicolor/*/apps/db-modeling.png
%{_datadir}/icons/hicolor/scalable/apps/db-modeling.svg

%changelog
* Sun Nov 01 2026 smovisk <smovisk@local> - ${VERSION}-1
- Empacotamento RPM oficial para Fedora
EOF

echo "==> Gerando pacote RPM com rpmbuild..."
rpmbuild --define "_topdir $TOPDIR" -bb "$TOPDIR/SPECS/db-modeling.spec"

cp "$TOPDIR"/RPMS/*/*.rpm "$DISTDIR"/

echo ""
echo "=========================================================="
echo " Pacote RPM gerado com sucesso!"
echo " Localização:"
ls -lh "$DISTDIR"/*.rpm
echo ""
echo " Para instalar ou atualizar no Fedora via DNF:"
echo "   sudo dnf install ./dist/${NAME}-${VERSION}-1.*.rpm"
echo "   sudo dnf upgrade ./dist/${NAME}-${VERSION}-1.*.rpm"
echo "=========================================================="
