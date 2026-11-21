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
programs: it hands back only memory it frees itself (`mi_free`), and a
parser panic is caught inside the library and reported as the JSON's
`error`, so nothing unwinds across the C ABI. `tests/smoke.c` checks exactly that from a
`cl`-built caller in CI.

## Releases

Tag `capi-vX.Y.Z` and push it. `.github/workflows/capi-release.yml` runs the
tests and builds every target on Linux. It smoke-tests native loading on
Apple Silicon, Windows x64 and ARM64, and Linux x64 and ARM64. Intel macOS
is still built and packaged but has no native smoke test in CI. The release
includes `mediainfo-capi-X.Y.Z-<target>.tar.gz` and its `.sha256` for every target.
