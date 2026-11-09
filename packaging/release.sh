#!/bin/sh
# Builds ZTBrowser release packages into dist/, with SHA256SUMS.
#
#   arch      pacman package, built here with makepkg (makepkg.conf flags apply: for this machine); uses Arch's Qt
#   appimage  built in a Fedora 44 container with its own Qt, QtWebEngine and QML modules inside; runs on any
#             x86_64 Linux whose glibc is as new as Fedora 44's, with the graphics stack and a desktop portal
#             from the host
#   rpm       built in a Fedora 44 container; uses Fedora's Qt and QtWebEngine
#
# There is no .deb: no stable Debian or Ubuntu ships a Qt WebEngine whose Chromium meets the engine floor
# (MinimumEngineVersion::CURRENT_FLOOR), and a package that starts with a security warning is not a release.
#
# Usage: packaging/release.sh [arch] [appimage] [rpm]      (no argument: all of them)
# Containers run with rootless podman; CONTAINER_ENGINE=docker works too (the files it creates are handed back).
set -eu

repository=$(cd "$(dirname "$0")/.." && pwd -P)
engine=${CONTAINER_ENGINE:-podman}
fedora_image=registry.fedoraproject.org/fedora:44
version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$repository/Cargo.toml" | head -n 1)
formats=${*:-arch appimage rpm}
work=$repository/target/containers
mkdir -p "$repository/dist" "$work"

# Runs a command in a build container: the repository read-only at /src, dist/ at /dist, scratch at /work,
# and a named volume that keeps downloaded crates between builds.
in_container() {
  image=$1 crates_volume=$2 crates_directory=$3
  shift 3
  # Rootless podman maps the container's root to the person running it; docker needs the files handed back.
  owner=""
  if [ "$engine" = docker ]; then owner="$(id -u):$(id -g)"; fi
  "$engine" run --rm --env "HOST_OWNER=$owner" \
    --volume "$repository:/src:ro,z" --volume "$repository/dist:/dist:z" --volume "$work:/work:z" \
    --volume "$crates_volume:$crates_directory" \
    --env "VERSION=$version" \
    "$image" "$@"
}

for format in $formats; do
  case $format in
    arch)
      (cd "$repository/packaging/arch" &&
        BUILDDIR="$repository/target/makepkg" PKGDEST="$repository/dist" makepkg --syncdeps --force)
      ;;
    appimage)
      in_container "$fedora_image" ztbrowser-crates-fedora /root/.cargo/registry sh /src/packaging/appimage/build.sh
      ;;
    rpm)
      git -C "$repository" ls-files -z --cached --others --exclude-standard |
        tar --create --gzip --file="$work/ztbrowser-$version.tar.gz" --transform="s,^,ztbrowser-$version/," \
          --directory="$repository" --null --files-from=-
      in_container "$fedora_image" ztbrowser-crates-fedora /root/.cargo/registry sh /src/packaging/rpm/build.sh
      ;;
    *)
      echo "unknown format: $format (expected arch, appimage or rpm)" >&2
      exit 2
      ;;
  esac
done

(cd "$repository/dist" && sha256sum -- *.rpm *.AppImage *.pkg.tar.* 2>/dev/null > SHA256SUMS || true)
echo "packages in $repository/dist:"
ls -1 "$repository/dist"
