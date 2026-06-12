use crate::cursor::Cursor;
use crate::error::{Error, Result};
use super::parser_jpeg::read_jpeg_segment;

fn should_keep_segment(marker: u8) -> bool {
    matches!(
        marker,
        0xD8 | 0xD9 | 0xE0 | 0xDB | 0xC0 | 0xC1 | 0xC2 | 0xC4 | 0xDA | 0xDD | 0xD0..=0xD7 | 0xE2 | 0xEE
    )
}

pub fn sanitize_jpeg(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err(Error::InvalidSignature("JPEG"));
    }

    let mut clean_jpeg = Vec::new();
    let mut cur = Cursor::new(data);

    loop {
        match read_jpeg_segment(&mut cur) {
            Some(segment) => {
                if should_keep_segment(segment.marker) {
                    clean_jpeg.push(0xFF);
                    clean_jpeg.push(segment.marker);

                    if !segment.data.is_empty() {
                        let length = (segment.data.len() + 2) as u16;
                        clean_jpeg.push((length >> 8) as u8);
                        clean_jpeg.push((length & 0xFF) as u8);
                        clean_jpeg.extend_from_slice(&segment.data);
                    }

                    if segment.marker == 0xDA {
                        let remaining = cur.remaining();
                        if remaining > 0 {
                            let compressed_data = &data[data.len() - remaining..];
                            clean_jpeg.extend_from_slice(compressed_data);
                        }
                        break;
                    }
                }

                if segment.marker == 0xD9 {
                    break;
                }
            }
            None => {
                let remaining = cur.remaining();
                if remaining > 0 {
                    let rest = &data[data.len() - remaining..];
                    clean_jpeg.extend_from_slice(rest);
                }
                break;
            }
        }
    }

    Ok(clean_jpeg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_keep_segment() {
        assert!(should_keep_segment(0xD8));
        assert!(should_keep_segment(0xD9));
        assert!(should_keep_segment(0xE0));
        assert!(!should_keep_segment(0xE1));
        assert!(!should_keep_segment(0xFE));
    }

    #[test]
    fn test_sanitize_jpeg_invalid() {
        let invalid = b"NOT A JPEG";
        assert!(sanitize_jpeg(invalid).is_err());
    }

    #[test]
    fn test_sanitize_jpeg_minimal() {
        let jpeg = vec![0xFF, 0xD8, 0xFF, 0xD9];
        let result = sanitize_jpeg(&jpeg);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), jpeg);
    }

    #[test]
    fn test_sanitize_jpeg_removes_exif() {
        let mut jpeg = vec![0xFF, 0xD8];
        jpeg.extend_from_slice(&[0xFF, 0xE1, 0x00, 0x04, 0xAA, 0xBB]);
        jpeg.extend_from_slice(&[0xFF, 0xD9]);

        let result = sanitize_jpeg(&jpeg).unwrap();
        assert!(!result.windows(2).any(|w| w == [0xFF, 0xE1]));
    }
}
