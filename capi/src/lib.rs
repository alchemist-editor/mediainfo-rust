//! C ABI over mediainfo-rust (`include/mediainfo_capi.h`): every field of an
//! input read through the caller's range-read callback, as one JSON document.
//! Built as a static library for native hosts and for wasm32.

use mediainfo::{MediaInfo, StreamKind};
use std::ffi::c_void;

pub const ABI: u32 = 1;

pub type ReadFn = unsafe extern "C" fn(ctx: *mut c_void, offset: u64, buf: *mut u8, len: usize) -> usize;

#[no_mangle]
pub extern "C" fn mi_capi_abi_version() -> u32 {
    ABI
}

/// The JSON report of `mi_extract` for an analysed handle.
pub fn fields_json(mi: &MediaInfo, recognised: bool) -> String {
    let mut out = String::from("{\"recognised\":");
    out.push_str(if recognised { "true" } else { "false" });
    out.push_str(",\"fields\":[");
    let mut first = true;
    for kind in StreamKind::ALL {
        for (index, stream) in mi.streams(kind).iter().enumerate() {
            for (key, value, _, _) in stream.fields() {
                if !first {
                    out.push(',');
                }
                first = false;
                out.push_str("{\"stream\":");
                string(&mut out, kind.name());
                out.push_str(",\"index\":");
                out.push_str(&index.to_string());
                out.push_str(",\"key\":");
                string(&mut out, key);
                out.push_str(",\"value\":");
                string(&mut out, value);
                out.push('}');
            }
        }
    }
    out.push_str("]}");
    out
}

fn string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

struct Ctx(*mut c_void);
// The callback and its context stay on the calling thread for the whole call.
unsafe impl Send for Ctx {}

/// # Safety
/// `read` must be valid to call with `ctx` for the duration of the call;
/// `name` must point at `name_len` readable bytes or be null; `out_len` must
/// be writable.
#[no_mangle]
pub unsafe extern "C" fn mi_extract(
    size: u64,
    read: ReadFn,
    ctx: *mut c_void,
    name: *const u8,
    name_len: usize,
    out_len: *mut usize,
) -> *mut u8 {
    let name = if name.is_null() || name_len == 0 {
        None
    } else {
        Some(String::from_utf8_lossy(std::slice::from_raw_parts(name, name_len)).into_owned())
    };
    let ctx = Ctx(ctx);
    let mut mi = MediaInfo::new();
    let recognised = mi.open_reader(
        size,
        Box::new(move |offset, buf: &mut [u8]| {
            let c = &ctx;
            read(c.0, offset, buf.as_mut_ptr(), buf.len()).min(buf.len())
        }),
        name.as_deref(),
    );
    let bytes = fields_json(&mi, recognised).into_bytes().into_boxed_slice();
    *out_len = bytes.len();
    Box::into_raw(bytes) as *mut u8
}

/// # Safety
/// `ptr` and `len` must come from one `mi_extract` result, freed once.
#[no_mangle]
pub unsafe extern "C" fn mi_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn read_vec(ctx: *mut c_void, offset: u64, buf: *mut u8, len: usize) -> usize {
        let data = &*(ctx as *const Vec<u8>);
        let start = (offset as usize).min(data.len());
        let n = len.min(data.len() - start);
        std::ptr::copy_nonoverlapping(data.as_ptr().add(start), buf, n);
        n
    }

    #[test]
    fn extracts_fields_through_the_callback() {
        let mut data = std::fs::read("../tests/fixtures/mpeg2_pcm.mxf").unwrap();
        let name = b"clip.mxf";
        let mut len = 0usize;
        let json = unsafe {
            let p = mi_extract(data.len() as u64, read_vec, &mut data as *mut Vec<u8> as *mut c_void, name.as_ptr(), name.len(), &mut len);
            let s = String::from_utf8(std::slice::from_raw_parts(p, len).to_vec()).unwrap();
            mi_free(p, len);
            s
        };
        assert!(json.starts_with("{\"recognised\":true,"), "{json}");
        assert!(json.contains("{\"stream\":\"General\",\"index\":0,\"key\":\"Format\",\"value\":\"MXF\"}"), "{json}");
        assert!(json.contains("\"stream\":\"Video\",\"index\":0,\"key\":\"Width\",\"value\":\"64\""), "{json}");
    }

    #[test]
    fn escapes_strings() {
        let mut s = String::new();
        string(&mut s, "a\"b\\c\nd\u{1}");
        assert_eq!(s, "\"a\\\"b\\\\c\\nd\\u0001\"");
    }
}
