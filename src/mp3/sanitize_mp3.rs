use crate::error::{Error, Result};
use super::parser_mp3::{has_id3v2, has_id3v1, has_ape_tag, is_mp3_signature};

fn get_id3v2_total_size(data: &[u8]) -> usize {
    if !has_id3v2(data) {
        return 0;
    }
    let size = ((data[6] as usize & 0x7F) << 21)
        | ((data[7] as usize & 0x7F) << 14)
        | ((data[8] as usize & 0x7F) << 7)
        | (data[9] as usize & 0x7F);
    10 + size
}

fn find_ape_tag(data: &[u8]) -> Option<(usize, usize)> {
    if data.len() < 32 {
        return None;
    }

    let search_end = if has_id3v1(data) { data.len() - 128 } else { data.len() };
    let search_start = search_end.saturating_sub(32);

    for i in (search_start..search_end).rev() {
        if i + 8 <= data.len() && &data[i..i + 8] == b"APETAGEX"
            && i + 16 <= data.len() {
            let size = (data[i + 12] as usize)
                | ((data[i + 13] as usize) << 8)
                | ((data[i + 14] as usize) << 16)
                | ((data[i + 15] as usize) << 24);
            let tag_size = size + 32;
            let tag_start = i.saturating_sub(size);
            return Some((tag_start, tag_size));
        }
    }
    None
}

pub fn sanitize_mp3(data: &[u8]) -> Result<Vec<u8>> {
    if !is_mp3_signature(data) {
        return Err(Error::InvalidSignature("MP3"));
    }

    let audio_start = get_id3v2_total_size(data);
    let mut audio_end = data.len();

    if has_id3v1(data) {
        audio_end -= 128;
    }

    if let Some((ape_start, _)) = find_ape_tag(data)
        && ape_start < audio_end && ape_start >= audio_start {
        audio_end = ape_start;
    }

    if audio_start >= audio_end {
        return Err(Error::NoAudioData);
    }

    Ok(data[audio_start..audio_end].to_vec())
}

pub fn create_empty_id3v2() -> Vec<u8> {
    vec![b'I', b'D', b'3', 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
}

pub fn verify_sanitization(data: &[u8]) -> bool {
    !has_id3v2(data) && !has_id3v1(data) && !has_ape_tag(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_id3v2_total_size() {
        let data = b"ID3\x04\x00\x00\x00\x00\x00\x10rest";
        assert_eq!(get_id3v2_total_size(data), 10 + 16);
    }

    #[test]
    fn test_get_id3v2_total_size_no_tag() {
        let data = b"\xFF\xFBrest";
        assert_eq!(get_id3v2_total_size(data), 0);
    }

    #[test]
    fn test_create_empty_id3v2() {
        let tag = create_empty_id3v2();
        assert_eq!(tag.len(), 10);
        assert_eq!(&tag[0..3], b"ID3");
    }

    #[test]
    fn test_sanitize_mp3_invalid() {
        let invalid = b"NOT AN MP3";
        assert!(sanitize_mp3(invalid).is_err());
    }

    #[test]
    fn test_sanitize_mp3_frame_sync() {
        let data = [0xFF, 0xFB, 0x90, 0x00, 0x00, 0x00];
        let result = sanitize_mp3(&data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_verify_sanitization() {
        let clean = [0xFF, 0xFB, 0x90, 0x00];
        assert!(verify_sanitization(&clean));
    }
}
