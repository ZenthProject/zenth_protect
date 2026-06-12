use crate::cursor::Cursor;
use super::segment_jpeg::JpegSegment;

/// Vérifie la signature JPEG
pub fn is_jpeg_signature(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    data[0] == 0xFF && data[1] == 0xD8
}

/// Lit un segment JPEG depuis le cursor
pub fn read_jpeg_segment(cur: &mut Cursor) -> Option<JpegSegment> {
    let prefix = cur.read_u8()?;
    if prefix != 0xFF {
        return None;
    }

    let marker = cur.read_u8()?;

    if marker == 0xD8 || marker == 0xD9 || marker == 0x01
       || (marker >= 0xD0 && marker <= 0xD7) {
        return Some(JpegSegment::new(marker, Vec::new()));
    }    
    let length = cur.read_u16_be()?;

    if length < 2 {
        return None;
    }

    let data_size = (length - 2) as usize;

    if cur.remaining() < data_size {
        return None;
    }

    let data_slice = cur.read_bytes(data_size)?;

    let data = data_slice.to_vec();

    Some(JpegSegment::new(marker, data))

}

#[cfg(test)]
fn classify_segment_privacy(marker: u8) -> &'static str {
    match marker {
        // SEGMENTS ESSENTIELS
        0xD8 => "[KEEP] ESSENTIEL (debut image)",
        0xD9 => "[KEEP] ESSENTIEL (fin image)",
        0xE0 => "[KEEP] ESSENTIEL (JFIF - info basiques)",
        0xDB => "[KEEP] ESSENTIEL (tables quantification)",
        0xC0 => "[KEEP] ESSENTIEL (frame baseline)",
        0xC1 => "[KEEP] ESSENTIEL (frame extended)",
        0xC2 => "[KEEP] ESSENTIEL (frame progressilve)",
        0xC4 => "[KEEP] ESSENTIEL (tables Huffman)",
        0xDA => "[KEEP] ESSENTIEL (debut scan)",
        0xDD => "[KEEP] ESSENTIEL (restart interval)",
        0xD0..=0xD7 => "[KEEP] ESSENTIEL (restart marker)",

        0xE2 => "[KEEP] COULEUR (ICC Profile - preserve les couleurs)",
        0xEE => "[KEEP] COULEUR (Adobe colorspace - CMYK/RGB)",

        0xE1 => "[REMOVE] TRES DANGEREUX (EXIF - GPS/date/camera)",
        0xED => "[REMOVE] DANGEREUX (Photoshop IPTC - auteur/copyright)",
        0xFE => "[REMOVE] DANGEREUX (commentaire - peut contenir n'importe quoi)",

        0xE3..=0xEC | 0xEF => "[REMOVE] APP marker (metadata non standard)",

        _ => "[REMOVE] INCONNU (segment non standard)",
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_jpeg_signature() {
        assert!(is_jpeg_signature(&[0xFF, 0xD8, 0xFF, 0xE0]));
        assert!(!is_jpeg_signature(&[0xFF, 0x00]));
        assert!(!is_jpeg_signature(&[0xFF]));
        assert!(!is_jpeg_signature(&[]));
    }

    #[test]
    fn test_read_jpeg_segment_soi() {
        let data = [0xFF, 0xD8];
        let mut cur = Cursor::new(&data);
        let seg = read_jpeg_segment(&mut cur).unwrap();
        assert_eq!(seg.marker, 0xD8);
        assert!(seg.data.is_empty());
    }

    #[test]
    fn test_read_jpeg_segment_with_data() {
        let data = [0xFF, 0xE0, 0x00, 0x04, 0xAA, 0xBB];
        let mut cur = Cursor::new(&data);
        let seg = read_jpeg_segment(&mut cur).unwrap();
        assert_eq!(seg.marker, 0xE0);
        assert_eq!(seg.data, vec![0xAA, 0xBB]);
    }

    #[test]
    fn test_classify_segment_privacy() {
        assert!(classify_segment_privacy(0xD8).contains("KEEP"));
        assert!(classify_segment_privacy(0xE1).contains("DANGEREUX"));
        assert!(classify_segment_privacy(0xFE).contains("DANGEREUX"));
    }
}

