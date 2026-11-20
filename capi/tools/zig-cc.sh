#!/bin/sh
# Linker for cross-building the shared library from a Linux host with Zig's
# clang, which carries its own macOS stubs, MinGW-w64 import libraries, libc,
# compiler-rt and libunwind. ZIG_CC_TARGET names the Zig target, for example
# aarch64-macos or x86_64-windows-gnu.
#
# rustc passes some flags written for Apple's or the GNU toolchain that zig cc
# either supplies itself or spells differently: the architecture comes from
# the target, a module-definition file is a plain input rather than `-Wl,`,
# and the runtime libraries are Zig's own.
: "${ZIG_CC_TARGET:?set ZIG_CC_TARGET}"
synchronization_lib() {
  dir="${TMPDIR:-/tmp}/zig-cc-$(id -u)"
  case "$ZIG_CC_TARGET" in aarch64-*) machine=arm64 ;; *) machine=i386:x86-64 ;; esac
  lib="$dir/libsynchronization-$machine.a"
  if [ ! -f "$lib" ]; then
    mkdir -p "$dir"
    printf 'LIBRARY api-ms-win-core-synch-l1-2-0.dll\nEXPORTS\nWaitOnAddress\nWakeByAddressAll\nWakeByAddressSingle\n' >"$dir/synchronization.def"
    "${ZIG:-zig}" dlltool -m "$machine" -d "$dir/synchronization.def" -l "$lib.tmp.$$" && mv "$lib.tmp.$$" "$lib"
  fi
  printf '%s' "$lib"
}
out=""
skip_next=""
for a in "$@"; do
  if [ -n "$skip_next" ]; then skip_next=""; continue; fi
  case "$a" in
    -arch) skip_next=1 ;;
    -nodefaultlibs|-Wl,-dead_strip) ;;
    # The libc crate links libiconv on Apple targets for declarations nothing
    # here calls, and Zig's macOS stubs carry libSystem alone: an iconv
    # symbol actually used would still fail the link as undefined.
    -liconv) ;;
    -Wl,*.def) out="$out '${a#-Wl,}'" ;;
    -nolibc|--unwindlib=*|-static-libgcc|-lmingw32|-lmingwex|-lmsvcrt|-lgcc_eh|-lgcc|-l:libpthread.a) ;;
    # Newer Rust std links WaitOnAddress and friends from synchronization,
    # an import library Zig's MinGW-w64 set lacks: make it from its .def.
    -lsynchronization) out="$out '$(synchronization_lib)'" ;;
    *) out="$out '$(printf '%s' "$a" | sed "s/'/'\\\\''/g")'" ;;
  esac
done
eval exec "${ZIG:-zig}" cc -target "$ZIG_CC_TARGET" $out
