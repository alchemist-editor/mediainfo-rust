/* Loads the shared library from a program built by the platform's own
 * compiler (MSVC on Windows, the system cc elsewhere) and reads one file
 * through the range-read callback. Proves the Zig-linked library and an
 * MSVC-built caller agree on the C ABI: calls, the callback, and memory the
 * library frees itself.
 *
 *   smoke <media file>     exits 0 when the file is recognised with fields */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "mediainfo_capi.h"

static size_t read_file(void *ctx, uint64_t offset, uint8_t *buf, size_t len) {
  FILE *f = (FILE *)ctx;
#ifdef _WIN32
  if (_fseeki64(f, (long long)offset, SEEK_SET) != 0) return 0;
#else
  if (fseeko(f, (off_t)offset, SEEK_SET) != 0) return 0;
#endif
  return fread(buf, 1, len, f);
}

int main(int argc, char **argv) {
  if (argc != 2) { fprintf(stderr, "usage: smoke <file>\n"); return 2; }
  if (mi_capi_abi_version() != MI_CAPI_ABI) { fprintf(stderr, "ABI mismatch\n"); return 1; }
  FILE *f = fopen(argv[1], "rb");
  if (!f) { perror(argv[1]); return 1; }
#ifdef _WIN32
  _fseeki64(f, 0, SEEK_END);
  uint64_t size = (uint64_t)_ftelli64(f);
#else
  fseeko(f, 0, SEEK_END);
  uint64_t size = (uint64_t)ftello(f);
#endif
  size_t len = 0;
  uint8_t *json = mi_extract(size, read_file, f, (const uint8_t *)argv[1], strlen(argv[1]), &len);
  fclose(f);
  if (!json) { fprintf(stderr, "mi_extract returned NULL\n"); return 1; }
  printf("%.*s\n", (int)(len < 400 ? len : 400), (const char *)json);
  int ok = len > 0 && strstr((const char *)json, "\"recognised\":true") != NULL &&
           strstr((const char *)json, "\"key\":") != NULL;
  mi_free(json, len);
  return ok ? 0 : 1;
}
