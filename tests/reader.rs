//! `open_reader` reads through a caller's callback and reports exactly what
//! `open` reports for the same bytes, while asking only for the windows the
//! parsers need.

use std::cell::Cell;
use std::fs;
use std::rc::Rc;

fn fixtures() -> Vec<std::path::PathBuf> {
    let mut v: Vec<_> = fs::read_dir("tests/fixtures")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e != "sh"))
        .collect();
    v.sort();
    v
}

#[test]
fn callback_reports_match_path_reports() {
    let all = fixtures();
    assert!(all.len() > 50, "found {} fixtures", all.len());
    for path in all {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let data = Rc::new(fs::read(&path).unwrap());
        let mut by_bytes = mediainfo::MediaInfo::new();
        let want = by_bytes.open_bytes(data.to_vec(), Some(&name));
        let src = data.clone();
        let mut by_reader = mediainfo::MediaInfo::new();
        let got = by_reader.open_reader(
            data.len() as u64,
            Box::new(move |off, buf| {
                let start = (off as usize).min(src.len());
                let n = buf.len().min(src.len() - start);
                buf[..n].copy_from_slice(&src[start..start + n]);
                n
            }),
            Some(&name),
        );
        assert_eq!(got, want, "{name}: recognised");
        assert_eq!(by_reader.inform(), by_bytes.inform(), "{name}: report");
    }
}

#[test]
fn callback_reads_a_large_input_by_window() {
    // A 1 GiB input that is all zeros past a tiny MP4 header: the parsers must
    // not read it whole.
    let head = fs::read("tests/fixtures/h264_aac.mp4").unwrap();
    let len = 1u64 << 30;
    let asked = Rc::new(Cell::new(0u64));
    let seen = asked.clone();
    let mut mi = mediainfo::MediaInfo::new();
    mi.open_reader(
        len,
        Box::new(move |off, buf| {
            seen.set(seen.get() + buf.len() as u64);
            for (i, b) in buf.iter_mut().enumerate() {
                let at = off as usize + i;
                *b = if at < head.len() { head[at] } else { 0 };
            }
            buf.len()
        }),
        Some("big.mp4"),
    );
    assert!(asked.get() < 64 << 20, "read {} bytes of a 1 GiB input", asked.get());
}

#[test]
fn a_short_callback_is_an_io_error() {
    let data = fs::read("tests/fixtures/h264_aac.mp4").unwrap();
    let len = data.len() as u64 * 4; // claims more than it serves
    let mut mi = mediainfo::MediaInfo::new();
    let result = mi.try_open_reader(
        len,
        Box::new(move |off, buf| {
            let start = (off as usize).min(data.len());
            let n = buf.len().min(data.len() - start);
            buf[..n].copy_from_slice(&data[start..start + n]);
            n
        }),
        Some("short.mp4"),
    );
    match result {
        Err(mediainfo::OpenError::Io(e)) => assert_eq!(e.kind(), std::io::ErrorKind::UnexpectedEof),
        other => panic!("expected UnexpectedEof, got {other:?}"),
    }
    assert!(!mi.is_open());
}
