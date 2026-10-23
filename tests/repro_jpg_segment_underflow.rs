use little_exif::metadata::Metadata;
use little_exif::filetype::FileExtension;

// FFD8 (SOI) + FF E1 00 01 (APP1 marker, length field = 1 < 2).
// clear_segment computes `(length - 2) as usize` without the `length < 2`
// guard that generic_read_metadata has, so u16 1 - 2 underflows.
#[test]
fn repro_app_segment_length_below_two() {
    let mut buf: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xE1, 0x00, 0x01];
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = Metadata::clear_metadata(&mut buf, FileExtension::JPEG);
    }));
    assert!(r.is_ok(), "clear_metadata panicked on a segment length < 2");
}
