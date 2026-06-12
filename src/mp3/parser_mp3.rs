use super::metadata_mp3::{Id3v1Metadata, Id3v2Metadata, ID3V1_GENRES};

pub fn has_id3v2(data: &[u8]) -> bool {
    if data.len() < 10 {
        return false;
    }
    data[0] == b'I' && data[1] == b'D' && data[2] == b'3'
}

pub fn has_id3v1(data: &[u8]) -> bool {
    if data.len() < 128 {
        return false;
    }
    let pos = data.len() - 128;
    data[pos] == b'T' && data[pos + 1] == b'A' && data[pos + 2] == b'G'
}

pub fn is_mp3_signature(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }

    if has_id3v2(data) {
        return true;
    }

    if has_id3v1(data) {
        return true;
    }

    if data[0] == 0xFF && (data[1] & 0xE0) == 0xE0 {
        return true;
    }

    false
}

fn decode_syncsafe_int(bytes: &[u8]) -> u32 {
    if bytes.len() < 4 {
        return 0;
    }
    ((bytes[0] as u32 & 0x7F) << 21)
        | ((bytes[1] as u32 & 0x7F) << 14)
        | ((bytes[2] as u32 & 0x7F) << 7)
        | (bytes[3] as u32 & 0x7F)
}

fn decode_u32_be(bytes: &[u8]) -> u32 {
    if bytes.len() < 4 {
        return 0;
    }
    ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) << 8)
        | (bytes[3] as u32)
}

fn extract_string_from_frame(data: &[u8]) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    let encoding = data[0];
    let content = &data[1..];

    match encoding {
        0x00 => {
            let s: String = content
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect();
            if s.is_empty() { None } else { Some(s) }
        }
        0x01 | 0x02 => {
            let mut result = String::new();
            let start = if content.len() >= 2 && (content[0] == 0xFF || content[0] == 0xFE) {
                2
            } else {
                0
            };

            for chunk in content[start..].chunks(2) {
                if chunk.len() == 2 {
                    let ch = if content.get(0) == Some(&0xFF) {
                        chunk[0]
                    } else {
                        chunk[1]
                    };
                    if ch == 0 { break; }
                    if ch.is_ascii() {
                        result.push(ch as char);
                    }
                }
            }
            if result.is_empty() { None } else { Some(result) }
        }
        0x03 => {
            let end = content.iter().position(|&b| b == 0).unwrap_or(content.len());
            String::from_utf8(content[..end].to_vec()).ok()
        }
        _ => {
            let s: String = content
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect();
            if s.is_empty() { None } else { Some(s) }
        }
    }
}

pub fn parse_id3v2(data: &[u8]) -> Option<Id3v2Metadata> {
    if !has_id3v2(data) {
        return None;
    }

    let mut metadata = Id3v2Metadata::new();

    metadata.version_major = data[3];
    metadata.version_minor = data[4];
    metadata.tag_size = decode_syncsafe_int(&data[6..10]);

    let mut pos: usize = 10;
    let end_pos = 10 + metadata.tag_size as usize;

    if end_pos > data.len() {
        return Some(metadata);
    }

    while pos + 10 <= end_pos {
        let frame_id = &data[pos..pos + 4];

        if frame_id[0] == 0 {
            break;
        }

        let frame_size = if metadata.version_major >= 4 {
            decode_syncsafe_int(&data[pos + 4..pos + 8]) as usize
        } else {
            decode_u32_be(&data[pos + 4..pos + 8]) as usize
        };

        if pos + 10 + frame_size > end_pos || frame_size == 0 {
            break;
        }

        let frame_data = &data[pos + 10..pos + 10 + frame_size];

        match frame_id {
            b"TIT2" => metadata.title = extract_string_from_frame(frame_data),
            b"TPE1" => metadata.artist = extract_string_from_frame(frame_data),
            b"TALB" => metadata.album = extract_string_from_frame(frame_data),
            b"TYER" | b"TDRC" => metadata.year = extract_string_from_frame(frame_data),
            b"TRCK" => metadata.track = extract_string_from_frame(frame_data),
            b"TCON" => metadata.genre = extract_string_from_frame(frame_data),
            b"COMM" => {
                if frame_data.len() > 4 {
                    let text_data = &frame_data[4..];
                    if let Some(null_pos) = text_data.iter().position(|&b| b == 0) {
                        if null_pos + 1 < text_data.len() {
                            let comment_bytes = &text_data[null_pos + 1..];
                            let mut temp = vec![frame_data[0]];
                            temp.extend_from_slice(comment_bytes);
                            metadata.comment = extract_string_from_frame(&temp);
                        }
                    }
                }
            }
            b"TCOM" => metadata.composer = extract_string_from_frame(frame_data),
            b"TENC" | b"TSSE" => metadata.encoder = extract_string_from_frame(frame_data),
            b"APIC" => metadata.has_picture = true,
            b"PRIV" => metadata.has_private_data = true,
            b"GEOB" => metadata.has_embedded_objects = true,
            b"TXXX" => {
                if frame_data.len() > 1 {
                    let encoding = frame_data[0];
                    let content = &frame_data[1..];

                    let null_pos = if encoding == 0x01 || encoding == 0x02 {
                        content.windows(2).position(|w| w == [0, 0]).map(|p| p + 2)
                    } else {
                        content.iter().position(|&b| b == 0).map(|p| p + 1)
                    };

                    if let Some(sep) = null_pos {
                        if sep < content.len() {
                            let desc_bytes = &content[..sep - 1];
                            let value_bytes = &content[sep..];

                            let mut desc_data = vec![encoding];
                            desc_data.extend_from_slice(desc_bytes);
                            let mut value_data = vec![encoding];
                            value_data.extend_from_slice(value_bytes);

                            if let (Some(desc), Some(value)) = (
                                extract_string_from_frame(&desc_data),
                                extract_string_from_frame(&value_data),
                            ) {
                                metadata.custom_fields.push((desc, value));
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        pos += 10 + frame_size;
    }

    Some(metadata)
}

pub fn parse_id3v1(data: &[u8]) -> Option<Id3v1Metadata> {
    if !has_id3v1(data) {
        return None;
    }

    let mut metadata = Id3v1Metadata::new();
    let tag_start = data.len() - 128;
    let tag = &data[tag_start..];

    let extract_field = |start: usize, len: usize| -> Option<String> {
        let field = &tag[start..start + len];
        let s: String = field
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect();
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
    };

    metadata.title = extract_field(3, 30);
    metadata.artist = extract_field(33, 30);
    metadata.album = extract_field(63, 30);
    metadata.year = extract_field(93, 4);
    metadata.comment = extract_field(97, 28);

    if tag[125] == 0 && tag[126] != 0 {
        metadata.track = Some(tag[126]);
    }

    let genre_id = tag[127];
    if (genre_id as usize) < ID3V1_GENRES.len() {
        metadata.genre_id = Some(genre_id);
    }

    Some(metadata)
}

pub fn has_ape_tag(data: &[u8]) -> bool {
    if data.len() < 32 {
        return false;
    }

    let search_start = if has_id3v1(data) {
        data.len().saturating_sub(128).saturating_sub(32)
    } else {
        data.len().saturating_sub(32)
    };

    if search_start >= data.len() {
        return false;
    }

    let search_area = &data[search_start..];
    search_area.windows(8).any(|w| w == b"APETAGEX")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_id3v2() {
        assert!(has_id3v2(b"ID3\x04\x00\x00\x00\x00\x00\x00"));
        assert!(!has_id3v2(b"NOTID3TAG"));
        assert!(!has_id3v2(b"ID3"));
    }

    #[test]
    fn test_has_id3v1() {
        let mut data = vec![0u8; 128];
        data[0] = b'T';
        data[1] = b'A';
        data[2] = b'G';
        assert!(has_id3v1(&data));
    }

    #[test]
    fn test_is_mp3_signature_id3v2() {
        let data = b"ID3\x04\x00\x00\x00\x00\x00\x00rest";
        assert!(is_mp3_signature(data));
    }

    #[test]
    fn test_is_mp3_signature_frame_sync() {
        let data = [0xFF, 0xFB, 0x90, 0x00];
        assert!(is_mp3_signature(&data));
    }

    #[test]
    fn test_is_mp3_signature_invalid() {
        assert!(!is_mp3_signature(b"NOTA"));
        assert!(!is_mp3_signature(b""));
    }

    #[test]
    fn test_decode_syncsafe_int() {
        assert_eq!(decode_syncsafe_int(&[0x00, 0x00, 0x02, 0x10]), 272);
        assert_eq!(decode_syncsafe_int(&[0x00, 0x00, 0x00, 0x7F]), 127);
    }

    #[test]
    fn test_decode_u32_be() {
        assert_eq!(decode_u32_be(&[0x00, 0x00, 0x01, 0x00]), 256);
    }

    #[test]
    fn test_has_ape_tag() {
        let mut data = vec![0u8; 64];
        data[32..40].copy_from_slice(b"APETAGEX");
        assert!(has_ape_tag(&data));
    }
}
