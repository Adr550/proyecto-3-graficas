//! Minimal lossless RGBA PNG writer. No image or GPU dependency is needed for headless renders.
use std::{fs, io, path::Path};
fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xffffffffu32;
    for &b in data {
        c ^= b as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xedb88320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
    }
    c ^ 0xffffffff
}
fn chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    let mut crc = Vec::with_capacity(data.len() + 4);
    crc.extend_from_slice(tag);
    crc.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc).to_be_bytes());
}
pub fn write_png(
    path: impl AsRef<Path>,
    width: usize,
    height: usize,
    rgba: &[u8],
) -> io::Result<()> {
    if rgba.len() != width * height * 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "RGBA length mismatch",
        ));
    }
    let mut raw = Vec::with_capacity(height * (width * 4 + 1));
    for row in rgba.chunks_exact(width * 4) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut z = vec![0x78, 0x01];
    let blocks = raw.len().div_ceil(65535);
    for (i, data) in raw.chunks(65535).enumerate() {
        z.push(if i + 1 == blocks { 1 } else { 0 });
        let n = data.len() as u16;
        z.extend_from_slice(&n.to_le_bytes());
        z.extend_from_slice(&(!n).to_le_bytes());
        z.extend_from_slice(data);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for x in &raw {
        a = (a + *x as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
    let mut header = Vec::new();
    header.extend_from_slice(&(width as u32).to_be_bytes());
    header.extend_from_slice(&(height as u32).to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &header);
    chunk(&mut out, b"IDAT", &z);
    chunk(&mut out, b"IEND", &[]);
    fs::write(path, out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn png_header() {
        let path = std::env::temp_dir().join("little-orbits-test.png");
        write_png(&path, 2, 2, &[255; 16]).unwrap();
        let b = fs::read(&path).unwrap();
        assert_eq!(&b[..8], b"\x89PNG\r\n\x1a\n");
        fs::remove_file(path).unwrap();
    }
}
