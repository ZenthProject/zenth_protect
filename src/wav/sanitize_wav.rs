use crate::error::{Error, Result};
use super::parser_wav::{is_wav_signature, list_chunks};
use super::chunk_wav::WavChunk;

fn write_u32_le(buffer: &mut [u8], offset: usize, value: u32) -> bool {
    if offset.checked_add(4).is_none_or(|end| end > buffer.len()) {
        return false;
    }
    buffer[offset] = value as u8;
    buffer[offset + 1] = (value >> 8) as u8;
    buffer[offset + 2] = (value >> 16) as u8;
    buffer[offset + 3] = (value >> 24) as u8;
    true
}

fn should_keep_chunk(chunk: &WavChunk) -> bool {
    matches!(
        &chunk.chunk_id,
        b"fmt " | b"FMT " | b"data" | b"DATA" | b"fact" | b"FACT"
    )
}

pub fn sanitize_wav(data: &[u8]) -> Result<Vec<u8>> {
    if !is_wav_signature(data) {
        return Err(Error::InvalidSignature("WAV"));
    }

    let chunks = list_chunks(data);
    let mut kept_size: u32 = 0;

    for chunk in &chunks {
        if should_keep_chunk(chunk) {
            let chunk_total = chunk.total_size();
            let padded = if chunk_total % 2 == 1 { chunk_total + 1 } else { chunk_total };
            kept_size += padded as u32;
        }
    }

    let new_file_size = 4 + kept_size;
    let mut result = Vec::with_capacity(8 + new_file_size as usize);

    result.extend_from_slice(b"RIFF");
    result.extend_from_slice(&[0, 0, 0, 0]);
    result.extend_from_slice(b"WAVE");

    for chunk in &chunks {
        if should_keep_chunk(chunk) {
            let chunk_start = chunk.offset;
            let chunk_total = chunk.total_size();

            if chunk_start + chunk_total <= data.len() {
                result.extend_from_slice(&data[chunk_start..chunk_start + chunk_total]);
                if chunk_total % 2 == 1 {
                    result.push(0);
                }
            }
        }
    }

    let file_size = (result.len() - 8) as u32;
    if !write_u32_le(&mut result, 4, file_size) {
        return Err(Error::WriteError);
    }

    Ok(result)
}

pub fn verify_sanitization(data: &[u8]) -> bool {
    if !is_wav_signature(data) {
        return false;
    }

    let chunks = list_chunks(data);

    for chunk in &chunks {
        if !should_keep_chunk(chunk) {
            return false;
        }
    }

    let has_fmt = chunks.iter().any(|c| c.chunk_id == *b"fmt " || c.chunk_id == *b"FMT ");
    let has_data = chunks.iter().any(|c| c.chunk_id == *b"data" || c.chunk_id == *b"DATA");

    has_fmt && has_data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_u32_le() {
        let mut buf = [0u8; 8];
        assert!(write_u32_le(&mut buf, 0, 0x12345678));
        assert_eq!(&buf[0..4], &[0x78, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_write_u32_le_out_of_bounds() {
        let mut buf = [0u8; 2];
        assert!(!write_u32_le(&mut buf, 0, 0x1234));
    }

    #[test]
    fn test_should_keep_chunk() {
        assert!(should_keep_chunk(&WavChunk::new(*b"fmt ", 0, 16)));
        assert!(should_keep_chunk(&WavChunk::new(*b"data", 0, 100)));
        assert!(should_keep_chunk(&WavChunk::new(*b"fact", 0, 4)));
        assert!(!should_keep_chunk(&WavChunk::new(*b"LIST", 0, 50)));
        assert!(!should_keep_chunk(&WavChunk::new(*b"id3 ", 0, 100)));
    }

    #[test]
    fn test_sanitize_wav_invalid() {
        let invalid = b"NOT A WAV";
        assert!(sanitize_wav(invalid).is_err());
    }

    #[test]
    fn test_sanitize_wav_minimal() {
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&36u32.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&[0u8; 16]);
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&0u32.to_le_bytes());

        assert!(sanitize_wav(&wav).is_ok());
    }

    #[test]
    fn test_verify_sanitization() {
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&36u32.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&[0u8; 16]);
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&0u32.to_le_bytes());

        assert!(verify_sanitization(&wav));
    }
}
