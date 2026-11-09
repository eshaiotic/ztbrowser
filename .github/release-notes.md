Packages for x86-64 Linux. ZTBrowser runs on QtWebEngine (Chromium) with a Qt Quick interface. The rpm and
pacman packages use your distribution's Qt and QtWebEngine, so the browser engine keeps receiving its security
updates; the AppImage carries its own Qt and QtWebEngine and gets engine updates with each ZTBrowser release.

| File | For |
|---|---|
| `ztbrowser-*.fc44.x86_64.rpm` | Fedora 44 |
| `ztbrowser-*-x86_64.pkg.tar.zst` | Arch Linux, CachyOS, EndeavourOS, Manjaro |
| `ZTBrowser-*-x86_64.AppImage` | any other distribution with glibc 2.43 or newer (large: it contains Chromium 140) |

**Start time: prefer the package.** The window of the rpm and pacman packages appears in about 0.7 s; the
AppImage needs about 1.7 s, because every launch mounts its compressed file system and unpacks Chromium on demand
(median of four runs to the first frame, KDE Plasma on Wayland, NVMe drive). Use the AppImage only where no package
exists for your distribution. An AppImage that put an entry into your application menu withdraws it at its next
start once a package is installed, so the menu then starts the package.

Check a download before installing it:

```sh
sha256sum --check --ignore-missing SHA256SUMS
gh attestation verify ZTBrowser-*-x86_64.AppImage --repo eshaiotic/ztbrowser
```

ZTBrowser is an independent project, not affiliated with or endorsed by Duck Duck Go, Inc.
