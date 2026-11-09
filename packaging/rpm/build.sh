#!/bin/sh
# Runs inside the Fedora build container started by packaging/release.sh: installs the spec's build
# dependencies and builds the RPM from /work/ztbrowser-<version>.tar.gz into /dist.
set -eu

# Containers started with docker leave root-owned files behind; hand them back to the person who ran the build.
if [ -n "${HOST_OWNER:-}" ]; then
  trap 'chown -R "$HOST_OWNER" /dist /work' EXIT
fi

dnf -y -q builddep /src/packaging/rpm/ztbrowser.spec >/dev/null
dnf -y -q install rpm-build >/dev/null

topdir=$(mktemp -d)
mkdir -p "$topdir/SOURCES"
cp "/work/ztbrowser-${VERSION}.tar.gz" "$topdir/SOURCES/"
rpmbuild --quiet -bb --define "_topdir $topdir" /src/packaging/rpm/ztbrowser.spec
for package in "$topdir"/RPMS/*/ztbrowser-*.rpm; do
  cp "$package" /dist/
  echo "built /dist/$(basename "$package")"
done
