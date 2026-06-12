#[cfg(test)]
use crate::cursor::Cursor;

pub fn is_png_signature(data: &[u8]) -> bool {
    data.len() >= 8 && data[..8] == *b"\x89PNG\r\n\x1A\n"
}

#[cfg(test)]
fn read_signature(cur: &mut Cursor) -> bool {
    let sig = cur.read_bytes(8);
    if sig.is_none() {
        return false;
    }
    let sig = sig.unwrap();
    sig == b"\x89PNG\r\n\x1A\n"
}

#[cfg(test)]
fn read_chunk_header(cur: &mut Cursor) -> Option<(u32, [u8; 4])> {
    let length = cur.read_u32_be()?;
    let chunk_type_bytes = cur.read_bytes(4)?;

    let chunk_type = [
        chunk_type_bytes[0],
        chunk_type_bytes[1],
        chunk_type_bytes[2],
        chunk_type_bytes[3],
    ];
    Some((length, chunk_type))
}

pub fn classify_chunk_privacy(chunk_type: &[u8; 4]) -> &'static str {
    match chunk_type {
        b"IHDR" => "[KEEP] ESSENTIEL (header)",
        b"IDAT" => "[KEEP] ESSENTIEL (donnees image)",
        b"IEND" => "[KEEP] ESSENTIEL (fin)",
        b"PLTE" => "[KEEP] ESSENTIEL (palette, si color type 3)",
        b"tRNS" => "[KEEP] UTILE (transparence)",
        b"tEXt" => "[REMOVE] DANGEREUX (texte non compresse - software/author)",
        b"zTXt" => "[REMOVE] DANGEREUX (texte compresse - metadata)",
        b"iTXt" => "[REMOVE] DANGEREUX (texte UTF-8 international - metadata)",
        b"tIME" => "[REMOVE] DANGEREUX (timestamp - revele date modification)",
        b"eXIf" => "[REMOVE] TRES DANGEREUX (EXIF - GPS/camera/date)",
        b"iCCP" => "[REMOVE] DANGEREUX (profil ICC - peut contenir device info)",
        b"sRGB" => "[KEEP] COULEUR (indicateur sRGB - 1 byte, preserve les couleurs)",
        b"gAMA" => "[KEEP] COULEUR (gamma - 4 bytes numeriques, preserve les couleurs)",
        b"cHRM" => "[KEEP] COULEUR (chromaticite - valeurs numeriques, preserve les couleurs)",
        b"pHYs" => "[REMOVE] OPTIONNEL (resolution DPI - peut reveler device)",
        b"sPLT" => "[REMOVE] OPTIONNEL (palette suggeree - inutile)",
        b"bKGD" => "[REMOVE] OPTIONNEL (couleur de fond - inutile)",
        b"hIST" => "[REMOVE] OPTIONNEL (histogramme - inutile)",
        b"sBIT" => "[REMOVE] OPTIONNEL (bits significatifs - inutile)",
        _ => "[REMOVE] INCONNU (chunk non standard - a supprimer par securite)",
    }
}

pub fn describe_color_type(color_type: u8) -> &'static str {
    match color_type {
        0 => "Grayscale (niveaux de gris)",
        2 => "Truecolor (RGB)",
        3 => "Indexed (palette)",
        4 => "Grayscale + Alpha",
        6 => "Truecolor + Alpha (RGBA)",
        _ => "INVALIDE (non defini dans la spec PNG)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_signature_valid() {
        let valid = b"\x89PNG\r\n\x1A\n";
        assert!(is_png_signature(valid));
    }

    #[test]
    fn test_png_signature_invalid() {
        assert!(!is_png_signature(b"NOTAPNG!"));
        assert!(!is_png_signature(b"\x89PNG"));
        assert!(!is_png_signature(b""));
    }

    #[test]
    fn test_read_signature() {
        let valid = b"\x89PNG\r\n\x1A\nrest";
        let mut cur = Cursor::new(valid);
        assert!(read_signature(&mut cur));
    }

    #[test]
    fn test_read_chunk_header() {
        let data = [
            0x00, 0x00, 0x00, 0x0D,
            b'I', b'H', b'D', b'R',
        ];
        let mut cur = Cursor::new(&data);
        let (len, chunk_type) = read_chunk_header(&mut cur).unwrap();
        assert_eq!(len, 13);
        assert_eq!(&chunk_type, b"IHDR");
    }

    #[test]
    fn test_classify_chunk_privacy() {
        assert!(classify_chunk_privacy(b"IHDR").contains("KEEP"));
        assert!(classify_chunk_privacy(b"IDAT").contains("KEEP"));
        assert!(classify_chunk_privacy(b"eXIf").contains("DANGEREUX"));
        assert!(classify_chunk_privacy(b"tEXt").contains("DANGEREUX"));
    }

    #[test]
    fn test_describe_color_type() {
        assert!(describe_color_type(0).contains("Grayscale"));
        assert!(describe_color_type(2).contains("RGB"));
        assert!(describe_color_type(6).contains("RGBA"));
        assert!(describe_color_type(99).contains("INVALIDE"));
    }
}
