/* C ABI over mediainfo-rust (capi/src/lib.rs). */
#ifndef MEDIAINFO_CAPI_H
#define MEDIAINFO_CAPI_H
#include <stddef.h>
#include <stdint.h>

#define MI_CAPI_ABI 1

/* Fill `buf` with up to `len` bytes from absolute `offset`; return how many
 * were written (fewer at end of input, 0 on error). */
typedef size_t (*mi_read_fn)(void *ctx, uint64_t offset, uint8_t *buf, size_t len);

uint32_t mi_capi_abi_version(void);

/* Every non-empty field of the input as UTF-8 JSON:
 *   {"recognised":true,"fields":[{"stream":"Video","index":0,"key":"Width","value":"960"},...]}
 * A read that comes up short of `size`, or a parser panic caught by the
 * library, adds "error":"<message>" and leaves "fields" empty.
 * `name` (may be NULL) is the file name, for the extension hint. Free the
 * result with mi_free(ptr, *out_len). NULL only when out of memory. */
uint8_t *mi_extract(uint64_t size, mi_read_fn read, void *ctx,
                    const uint8_t *name, size_t name_len, size_t *out_len);

void mi_free(uint8_t *ptr, size_t len);

#endif
