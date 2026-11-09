#!/bin/sh
# Runs inside the Fedora 44 build container started by packaging/release.sh. Builds the binary and assembles the
# AppImage from it (packaging/appimage/assemble.sh).
#
# Fedora supplies the engine the AppImage carries, and the engine must not be older than ZTBrowser's floor
# (MinimumEngineVersion::CURRENT_FLOOR). Debian 13 ships Qt 6.8 (Chromium 122), which is far below it.
set -eu

# The spec's build dependencies are the build's; the rest are the runtime pieces the AppImage copies (QML modules,
# QtWebEngineProcess, SVG and Wayland plugins) and the tools that pack the image.
dnf -y -q builddep /src/packaging/rpm/ztbrowser.spec >/dev/null
dnf -y -q install qt6-qtdeclarative qt6-qtsvg qt6-qtwayland qt6-qtwebchannel qt6-qtwebengine \
  squashfs-tools binutils curl >/dev/null

# Containers started with docker leave root-owned files behind; hand them back to the person who ran the build.
if [ -n "${HOST_OWNER:-}" ]; then
  trap 'chown -R "$HOST_OWNER" /dist /work' EXIT
fi

export CARGO_TARGET_DIR=/work/fedora-appimage
cargo build --release --locked -p ztbrowser-qt --manifest-path /src/Cargo.toml
sh /src/packaging/appimage/assemble.sh "$CARGO_TARGET_DIR/release/ztbrowser-qt"
