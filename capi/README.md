# mediainfo-capi

A C ABI over mediainfo-rust for Alchemist: every field of a file, read through
the caller's range-read callback (so a multi-gigabyte file is never loaded
whole), returned as JSON. See [include/mediainfo_capi.h](include/mediainfo_capi.h).

## Shared libraries

`tools/build-shared.sh <rust-target> [out-dir] [version]` builds and packages
one target. macOS and Windows targets cross-build from Linux or macOS with Zig
(0.17) as the linker, through `tools/zig-cc.sh`:

| Rust target | Library | Notes |
| --- | --- | --- |
| `aarch64-apple-darwin`, `x86_64-apple-darwin` | `libmediainfo_capi.dylib` | install name `@rpath/…`, macOS 11.0 floor |
| `x86_64-pc-windows-gnullvm`, `aarch64-pc-windows-gnullvm` | `mediainfo_capi.dll` | Universal C Runtime; `.lib` (MSVC) and `.dll.a` (MinGW) import libraries |
| `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` | `libmediainfo_capi.so` | host cc |

The Windows DLL needs no MSVC toolchain to build and loads into MSVC-built
programs: it hands back only memory it frees itself (`mi_free`) and aborts
rather than unwinding on a panic. `tests/smoke.c` checks exactly that from a
`cl`-built caller in CI.

## Releases

Tag `capi-vX.Y.Z` and push it. `.github/workflows/capi-release.yml` runs the
tests, builds every target on Linux, loads each library on its own platform,
and attaches `mediainfo-capi-X.Y.Z-<target>.tar.gz` (with a `.sha256`) to the
release.
