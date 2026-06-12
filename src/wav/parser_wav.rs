use super::chunk_wav::{WavChunk, WavMetadata, WavFormat};

pub fn is_wav_signature(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }
    if &data[0..4] != b"RIFF" {
        return false;
    }

    if &data[8..12] != b"WAVE" {
        return false;
    }

    true
}

fn read_u32_le(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 4 > data.len() {
        return None;
    }
    Some(
        (data[offset] as u32)
            | ((data[offset + 1] as u32) << 8)
            | ((data[offset + 2] as u32) << 16)
            | ((data[offset + 3] as u32) << 24),
    )
}

fn read_u16_le(data: &[u8], offset: usize) -> Option<u16> {
    if offset + 2 > data.len() {
        return None;
    }
    Some((data[offset] as u16) | ((data[offset + 1] as u16) << 8))
}

pub fn read_chunk(data: &[u8], offset: usize) -> Option<WavChunk> {
    if offset + 8 > data.len() {
        return None;
    }

    let mut chunk_id = [0u8; 4];
    chunk_id.copy_from_slice(&data[offset..offset + 4]);

    let data_size = read_u32_le(data, offset + 4)?;
    Some(WavChunk::new(chunk_id, offset, data_size))
}

pub fn list_chunks(data: &[u8]) -> Vec<WavChunk> {
    let mut chunks = Vec::new();

    if !is_wav_signature(data) {
        return chunks;
    }

    let mut offset = 12;

    while offset + 8 <= data.len() {
        if let Some(chunk) = read_chunk(data, offset) {
            let chunk_size = chunk.total_size();
            chunks.push(chunk);

            let padded_size = if chunk_size % 2 == 1 {
                chunk_size + 1
            } else {
                chunk_size
            };

            offset += padded_size;
        } else {
            break;
        }
    }

    chunks
}

pub fn parse_fmt_chunk(data: &[u8], chunk: &WavChunk) -> Option<WavFormat> {
    let start = chunk.offset + 8;

    if start + 16 > data.len() {
        return None;
    }

    Some(WavFormat {
        audio_format: read_u16_le(data, start)?,
        num_channels: read_u16_le(data, start + 2)?,
        sample_rate: read_u32_le(data, start + 4)?,
        byte_rate: read_u32_le(data, start + 8)?,
        block_align: read_u16_le(data, start + 12)?,
        bits_per_sample: read_u16_le(data, start + 14)?,
    })
}

fn extract_string(data: &[u8], offset: usize, max_len: usize) -> Option<String> {
    if offset >= data.len() {
        return None;
    }

    let end = std::cmp::min(offset + max_len, data.len());
    let bytes = &data[offset..end];

    let null_pos = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let s = String::from_utf8_lossy(&bytes[..null_pos]).to_string();
    let trimmed = s.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_list_info(data: &[u8], chunk: &WavChunk, metadata: &mut WavMetadata) {
    let start = chunk.offset + 8;
    let end = start + chunk.data_size as usize;

    if start + 4 > data.len() || end > data.len() {
        return;
    }

    if &data[start..start + 4] != b"INFO" {
        return;
    }

    let mut pos = start + 4;

    while pos + 8 <= end {
        let sub_id = &data[pos..pos + 4];
        let sub_size = match read_u32_le(data, pos + 4) {
            Some(s) => s as usize,
            None => break,
        };

        let value_start = pos + 8;
        let value_end = std::cmp::min(value_start + sub_size, end);

        if value_start < data.len() && value_end <= data.len() {
            let value = extract_string(data, value_start, sub_size);

            match sub_id {
                b"IART" => metadata.artist = value,
                b"INAM" => metadata.title = value,
                b"IPRD" => metadata.album = value,
                b"ICRD" => metadata.creation_date = value,
                b"IGNR" => metadata.genre = value,
                b"ICMT" => metadata.comment = value,
                b"ISFT" => metadata.software = value,
                b"ICOP" => metadata.copyright = value,
                b"IENG" => metadata.engineer = value,
                b"ITCH" => metadata.technician = value,
                b"ISRC" => metadata.source = value,
                _ => {}
            }
        }

        let padded = if sub_size % 2 == 1 { sub_size + 1 } else { sub_size };
        pos += 8 + padded;
    }
}

pub fn extract_metadata(data: &[u8]) -> WavMetadata {
    let mut metadata = WavMetadata::new();
    let chunks = list_chunks(data);

    for chunk in &chunks {
        if chunk.is_metadata() {
            let id_str = chunk.id_string();
            if !metadata.metadata_chunks.contains(&id_str) {
                metadata.metadata_chunks.push(id_str);
            }
            metadata.total_metadata_size += chunk.data_size + 8;

            match &chunk.chunk_id {
                b"LIST" => parse_list_info(data, chunk, &mut metadata),
                b"id3 " | b"ID3 " => metadata.has_id3 = true,
                b"bext" | b"BEXT" => metadata.has_bext = true,
                b"_PMX" => metadata.has_xmp = true,
                _ => {}
            }
        }
    }

    metadata
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_wav_signature() {
        let valid = b"RIFF\x00\x00\x00\x00WAVEfmt ";
        assert!(is_wav_signature(valid));
    }

    #[test]
    fn test_is_wav_signature_invalid() {
        assert!(!is_wav_signature(b"NOTAWAVF"));
        assert!(!is_wav_signature(b"RIFF"));
        assert!(!is_wav_signature(b""));
    }

    #[test]
    fn test_read_u32_le() {
        assert_eq!(read_u32_le(&[0x10, 0x00, 0x00, 0x00], 0), Some(16));
        assert_eq!(read_u32_le(&[0x00, 0x00], 0), None);
    }

    #[test]
    fn test_read_u16_le() {
        assert_eq!(read_u16_le(&[0x10, 0x00], 0), Some(16));
        assert_eq!(read_u16_le(&[0x00], 0), None);
    }

    #[test]
    fn test_read_chunk() {
        let data = b"fmt \x10\x00\x00\x00";
        let chunk = read_chunk(data, 0).unwrap();
        assert_eq!(&chunk.chunk_id, b"fmt ");
        assert_eq!(chunk.data_size, 16);
    }

    #[test]
    fn test_list_chunks() {
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&36u32.to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&[0u8; 16]);
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&0u32.to_le_bytes());

        let chunks = list_chunks(&wav);
        assert_eq!(chunks.len(), 2);
    }
}
