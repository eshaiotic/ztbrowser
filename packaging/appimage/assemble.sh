#!/bin/sh
# Assembles ZTBrowser-<version>-x86_64.AppImage in /dist from the binary given as the only argument.
# Runs inside the Fedora build container (packaging/appimage/build.sh); VERSION comes from packaging/release.sh.
#
# An AppImage is a small runtime program followed by a SquashFS image of the application directory; the
# runtime mounts the image and starts its AppRun. The runtime is pinned to a release and verified by hash.
#
# Unlike the packages, this one carries its engine: Qt 6, QtWebEngine (Chromium) and the QML modules the
# window uses are copied from the build container. The image therefore runs on any x86_64 Linux whose glibc is
# at least as new as the container's (see the README for the number), with only the graphics stack (Mesa,
# X11/Wayland) taken from the host.
# The price is that the engine's security updates arrive with a new ZTBrowser release, not with the distribution.
set -eu

runtime_url=https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64
runtime_sha256=2fca8b443c92510f1483a883f60061ad09b46b978b2631c807cd873a47ec260d

if [ "$(uname -m)" != x86_64 ]; then
  echo "AppImage: only x86_64 is supported (the pinned runtime is x86_64)" >&2
  exit 1
fi

binary=$1
work=$(mktemp -d)
image="/dist/ZTBrowser-${VERSION}-x86_64.AppImage"

# Where this distribution keeps Qt: the layout differs between distributions, qmake knows (Fedora also calls it
# qmake-qt6).
qt_query() { if command -v qmake6 >/dev/null; then qmake6 -query "$1"; else qmake-qt6 -query "$1"; fi; }
plugins=$(qt_query QT_INSTALL_PLUGINS)
qml=$(qt_query QT_INSTALL_QML)
libexec=$(qt_query QT_INSTALL_LIBEXECS)
data=$(qt_query QT_INSTALL_DATA)
translations=$(qt_query QT_INSTALL_TRANSLATIONS)

curl -fsSL --proto '=https' --tlsv1.2 -o "$work/runtime" "$runtime_url"
echo "$runtime_sha256  $work/runtime" | sha256sum --check --quiet

app=$work/ZTBrowser.AppDir
mkdir -p "$app/usr/bin" "$app/usr/lib/qt6/plugins" "$app/usr/lib/qt6/qml" "$app/usr/share/qt6/translations"
install -m0755 "$binary" "$app/usr/bin/ztbrowser"
install -m0755 "$libexec/QtWebEngineProcess" "$app/usr/lib/qt6/QtWebEngineProcess"
cp -a "$data/resources" "$app/usr/share/qt6/resources"
cp -a "$translations/qtwebengine_locales" "$app/usr/share/qt6/translations/qtwebengine_locales"

# Qt plugins the window needs: platforms (X11, Wayland, offscreen), image formats (SVG icons), TLS, input methods.
for plugin in platforms imageformats tls platforminputcontexts xcbglintegrations iconengines \
  wayland-decoration-client wayland-graphics-integration-client wayland-shell-integration; do
  if [ -d "$plugins/$plugin" ]; then cp -a "$plugins/$plugin" "$app/usr/lib/qt6/plugins/$plugin"; fi
done
# The desktop's own file chooser and colour scheme come through the portal; without it Qt draws its own dialog.
mkdir -p "$app/usr/lib/qt6/plugins/platformthemes"
if [ -e "$plugins/platformthemes/libqxdgdesktopportal.so" ]; then
  cp "$plugins/platformthemes/libqxdgdesktopportal.so" "$app/usr/lib/qt6/plugins/platformthemes/"
fi
# QML modules: Qt Quick with Controls, Layouts, Dialogs, Effects, Window, Templates, QtQml and the web view.
for module in QtQml QtQuick QtWebEngine QtWebChannel QtCore Qt; do
  if [ -d "$qml/$module" ]; then cp -a "$qml/$module" "$app/usr/lib/qt6/qml/$module"; fi
done
# Only the Controls styles that need nothing beyond Qt itself.
find "$app/usr/lib/qt6/qml/QtQuick/Controls" -mindepth 1 -maxdepth 1 -type d \
  ! -name Basic ! -name Fusion ! -name impl ! -name designer -exec rm -rf {} +

# Every shared library any of those files needs, except what must come from the host: the C library and the
# graphics stack. Libraries the copied ones need in turn are found by a second pass.
excluded='^(linux-vdso|ld-linux|libc|libm|libdl|libpthread|librt|libresolv|libutil|libanl|libnss_[a-z]+|libGL|libEGL|libGLX|libGLdispatch|libOpenGL|libvulkan|libdrm|libgbm|libwayland-egl|libwayland-client|libwayland-server|libwayland-cursor|libX11|libX11-xcb|libxcb|libXau|libXdmcp|libnvidia[a-z-]*|libcuda)\.'
copy_needed_libraries() {
  awk '/=> \// {print $3}' | sort -u |
    while read -r library; do
      name=$(basename "$library")
      printf '%s\n' "$name" | grep -Eq "$excluded" && continue
      [ -e "$app/usr/lib/$name" ] || cp -L "$library" "$app/usr/lib/$name"
    done
}
find "$app" -type f \( -name '*.so' -o -name '*.so.*' -o -name ztbrowser -o -name QtWebEngineProcess \) |
  while read -r file; do ldd "$file" 2>/dev/null; done | copy_needed_libraries
find "$app/usr/lib" -maxdepth 1 -type f -name '*.so*' |
  while read -r file; do ldd "$file" 2>/dev/null; done | copy_needed_libraries
strip --strip-unneeded "$app"/usr/lib/*.so* 2>/dev/null || true

install -Dm0755 /src/packaging/appimage/AppRun "$app/AppRun"
install -Dm0644 /src/packaging/linux/io.github.ztbrowser.ZTBrowser.desktop -t "$app"
install -Dm0644 /src/packaging/linux/io.github.ztbrowser.ZTBrowser.desktop -t "$app/usr/share/applications"
install -Dm0644 /src/packaging/linux/io.github.ztbrowser.ZTBrowser.metainfo.xml -t "$app/usr/share/metainfo"
install -Dm0644 /src/assets/icons/io.github.ztbrowser.ZTBrowser.svg -t "$app"
install -Dm0644 /src/assets/icons/io.github.ztbrowser.ZTBrowser.svg -t "$app/usr/share/icons/hicolor/scalable/apps"
install -Dm0644 /src/NOTICE /src/LICENSE -t "$app/usr/share/doc/ztbrowser"
ln -s io.github.ztbrowser.ZTBrowser.svg "$app/.DirIcon"

mksquashfs "$app" "$work/image.squashfs" -root-owned -noappend -comp zstd -quiet
cat "$work/runtime" "$work/image.squashfs" > "$image"
chmod 0755 "$image"
echo "built $image ($(du -h "$image" | cut -f1))"
