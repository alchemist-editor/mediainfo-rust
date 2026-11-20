#!/bin/sh
# Build mediainfo-capi as a shared library for one Rust target and stage it
# as a release archive. Every macOS and Windows target cross-builds from a
# Linux (or macOS) host with Zig as the linker (tools/zig-cc.sh): no Xcode,
# no Visual Studio. Linux targets build with the host's own cc.
#
#   tools/build-shared.sh <rust-target> [out-dir] [version]
#
# Writes <out-dir>/mediainfo-capi-<version>-<rust-target>.tar.gz and its
# .sha256. The archive holds lib/ (the library, plus an import library on
# Windows), include/mediainfo_capi.h, LICENSE and COMMIT.
#
# macOS: the install name is @rpath/libmediainfo_capi.dylib, and the floor is
# macOS 11.0 (Alchemist Pro's LSMinimumSystemVersion).
# Windows: x86_64/aarch64-pc-windows-gnullvm, against the Universal C Runtime
# (part of Windows 10 and later). The C ABI hands back only memory the DLL
# frees itself (mi_free) and catches a parser panic inside, so the DLL loads
# into an MSVC-built program like any other; lib/ carries both a MinGW
# import library (.dll.a) and an MSVC one (.lib), made by zig dlltool from
# the DLL's exports.
set -eu

target="${1:?usage: build-shared.sh <rust-target> [out-dir] [version]}"
here="$(cd "$(dirname "$0")/.." && pwd)"
out_dir="$(mkdir -p "${2:-$here/target/dist}" && cd "${2:-$here/target/dist}" && pwd)"
version="${3:-dev}"
zig="${ZIG:-zig}"
min_macos="11.0"

env_target="$(printf '%s' "$target" | tr 'a-z-' 'A-Z_')"
linker_var="CARGO_TARGET_${env_target}_LINKER"
case "$target" in
  aarch64-apple-darwin) zig_target="aarch64-macos.$min_macos" ;;
  x86_64-apple-darwin) zig_target="x86_64-macos.$min_macos" ;;
  x86_64-pc-windows-gnullvm) zig_target="x86_64-windows-gnu"; machine="i386:x86-64" ;;
  aarch64-pc-windows-gnullvm) zig_target="aarch64-windows-gnu"; machine="arm64" ;;
  *-unknown-linux-gnu) zig_target="" ;;
  *) echo "build-shared.sh: no recipe for $target" >&2; exit 2 ;;
esac

if [ -n "$zig_target" ]; then
  export ZIG="$zig" ZIG_CC_TARGET="$zig_target"
  export "$linker_var=$here/tools/zig-cc.sh"
fi
case "$target" in
  *-apple-darwin)
    export MACOSX_DEPLOYMENT_TARGET="$min_macos"
    export RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=-Wl,-install_name,@rpath/libmediainfo_capi.dylib" ;;
esac

(cd "$here" && cargo rustc --release --locked --lib --crate-type cdylib --target "$target")

built="$here/target/$target/release"
name="mediainfo-capi-$version-$target"
stage="$out_dir/$name"
rm -rf "$stage" && mkdir -p "$stage/lib" "$stage/include"
case "$target" in
  *-windows-gnullvm)
    cp "$built/mediainfo_capi.dll" "$built/libmediainfo_capi.dll.a" "$stage/lib/"
    # An MSVC import library from the DLL's export table.
    {
      echo "LIBRARY mediainfo_capi.dll"
      echo "EXPORTS"
      python3 "$here/tools/pe-exports.py" "$built/mediainfo_capi.dll"
    } > "$stage/lib/mediainfo_capi.def"
    grep -q '^mi_extract$' "$stage/lib/mediainfo_capi.def" || { echo "no exports read from the DLL" >&2; exit 1; }
    "$zig" dlltool -m "$machine" -d "$stage/lib/mediainfo_capi.def" -D mediainfo_capi.dll -l "$stage/lib/mediainfo_capi.lib"
    ;;
  *-apple-darwin) cp "$built/libmediainfo_capi.dylib" "$stage/lib/" ;;
  *-linux-gnu) cp "$built/libmediainfo_capi.so" "$stage/lib/" ;;
esac
cp "$here/include/mediainfo_capi.h" "$stage/include/"
cp "$here/../LICENSE" "$stage/"
git -C "$here" rev-parse HEAD > "$stage/COMMIT"
(cd "$out_dir" && tar czf "$name.tar.gz" "$name")
if command -v sha256sum >/dev/null 2>&1; then (cd "$out_dir" && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256")
else (cd "$out_dir" && shasum -a 256 "$name.tar.gz" > "$name.tar.gz.sha256"); fi
ls -l "$stage/lib"
