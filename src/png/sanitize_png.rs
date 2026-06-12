use crate::cursor::Cursor;
use crate::error::{Error, Result};

pub fn sanitize_png(data: &[u8]) -> Result<Vec<u8>> {
    let mut cur = Cursor::new(data);
    let mut output = Vec::new();

    let signature = cur.read_bytes(8).ok_or(Error::TruncatedFile)?;
    if signature != b"\x89PNG\r\n\x1A\n" {
        return Err(Error::InvalidSignature("PNG"));
    }
    output.extend_from_slice(signature);

    let mut chunk_count = 0;
    loop {
        if cur.remaining() < 8 {
            break;
        }

        let length = cur.read_u32_be().ok_or(Error::TruncatedFile)?;
        let chunk_type_bytes = cur.read_bytes(4).ok_or(Error::TruncatedFile)?;

        let chunk_type: [u8; 4] = [
            chunk_type_bytes[0],
            chunk_type_bytes[1],
            chunk_type_bytes[2],
            chunk_type_bytes[3],
        ];

        if cur.remaining() < (length as usize + 4) {
            return Err(Error::InvalidChunk("truncated"));
        }

        let chunk_data = cur.read_bytes(length as usize).ok_or(Error::TruncatedFile)?;
        let crc_bytes = cur.read_bytes(4).ok_or(Error::TruncatedFile)?;

        if should_keep_chunk(&chunk_type) {
            output.extend_from_slice(&length.to_be_bytes());
            output.extend_from_slice(&chunk_type);
            output.extend_from_slice(chunk_data);
            output.extend_from_slice(crc_bytes);
            chunk_count += 1;
        }

        if chunk_type == *b"IEND" {
            break;
        }
    }

    if chunk_count == 0 {
        return Err(Error::InvalidChunk("no valid chunks"));
    }

    Ok(output)
}

fn should_keep_chunk(chunk_type: &[u8; 4]) -> bool {
    matches!(
        chunk_type,
        b"IHDR" | b"IDAT" | b"IEND" | b"PLTE" | b"tRNS" | b"sRGB" | b"gAMA" | b"cHRM"
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_keep_chunk() {
        assert!(should_keep_chunk(b"IHDR"));
        assert!(should_keep_chunk(b"IDAT"));
        assert!(should_keep_chunk(b"IEND"));
        assert!(!should_keep_chunk(b"tEXt"));
        assert!(!should_keep_chunk(b"eXIf"));
    }

    #[test]
    fn test_sanitize_png_invalid() {
        let invalid = b"NOT A PNG FILE";
        assert!(sanitize_png(invalid).is_err());
    }

    #[test]
    fn test_sanitize_minimal_png() {
        let mut png = Vec::new();
        png.extend_from_slice(b"\x89PNG\r\n\x1A\n");
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&1u32.to_be_bytes());
        png.extend_from_slice(&1u32.to_be_bytes());
        png.push(8);
        png.push(0);
        png.push(0);
        png.push(0);
        png.push(0);
        png.extend_from_slice(&0x815467C7u32.to_be_bytes());
        png.extend_from_slice(&0u32.to_be_bytes());
        png.extend_from_slice(b"IEND");
        png.extend_from_slice(&0xAE426082u32.to_be_bytes());

        assert!(sanitize_png(&png).is_ok());
    }
}
