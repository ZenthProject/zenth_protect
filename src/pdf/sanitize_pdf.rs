use crate::error::{Error, Result};
use super::parser_pdf::is_pdf_signature;

pub fn sanitize_pdf(data: &[u8]) -> Result<Vec<u8>> {
    if !is_pdf_signature(data) {
        return Err(Error::InvalidSignature("PDF"));
    }

    // PDF 1.5+ utilisant uniquement des XRef streams (pas de "trailer" keyword)
    // On ne peut pas traiter ce format sans décompresser les streams.
    if rfind_bytes(data, b"trailer").is_none() {
        return Err(Error::UnsupportedFormat("PDF 1.5+ with XRef streams only"));
    }

    let mut result = data.to_vec();

    // Effacer le contenu du dictionnaire /Info
    if let Some(info_num) = find_info_object_number(&result) {
        remove_info_object(&mut result, info_num);
    }

    // Effacer la référence /Info dans le trailer
    remove_info_from_trailer(&mut result);

    // Effacer les blocs XMP
    remove_xmp_xpacket(&mut result);
    remove_xmpmeta(&mut result);

    Ok(result)
}

pub fn verify_sanitization(data: &[u8]) -> bool {
    find_bytes(data, b"<?xpacket begin").is_none()
        && find_bytes(data, b"<x:xmpmeta").is_none()
}

// --- Utilitaires bytes ---

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn rfind_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).rposition(|w| w == needle)
}

/// Remplace une plage par des espaces (même longueur — préserve les offsets XRef).
fn erase(data: &mut [u8], start: usize, end: usize) {
    if start < end && end <= data.len() {
        data[start..end].fill(b' ');
    }
}

// --- Recherche du numéro d'objet /Info ---

fn find_info_object_number(data: &[u8]) -> Option<usize> {
    let trailer_pos = rfind_bytes(data, b"trailer")?;
    let search_end = (trailer_pos + 2000).min(data.len());
    let search = &data[trailer_pos..search_end];

    let info_pos = find_bytes(search, b"/Info")?;
    let after_info = &search[info_pos + 5..];

    // Sauter les espaces
    let ws_end = after_info
        .iter()
        .position(|&b| !b.is_ascii_whitespace())
        .unwrap_or(0);
    let after_ws = &after_info[ws_end..];

    // Lire les chiffres
    let num_end = after_ws
        .iter()
        .position(|&b| !b.is_ascii_digit())
        .unwrap_or(after_ws.len());

    if num_end == 0 {
        return None;
    }

    std::str::from_utf8(&after_ws[..num_end])
        .ok()?
        .parse::<usize>()
        .ok()
}

// --- Parser de dictionnaire imbriqué ---

/// Trouve la position du `>>` fermant correspondant au `<<` dont les données
/// commencent à `start`. Gère correctement :
/// - les dictionnaires imbriqués `<< ... >>`
/// - les chaînes littérales `( ... )` avec parenthèses échappées
/// - les chaînes hex `< ... >` (à ne pas confondre avec `<<`)
/// - les commentaires `% ...`
fn find_dict_end(data: &[u8], start: usize) -> Option<usize> {
    let mut depth: i32 = 1;
    let mut i = start;

    while i < data.len() {
        match data[i] {
            b'(' => {
                // Chaîne littérale — ignorer son contenu
                i += 1;
                let mut paren: i32 = 1;
                while i < data.len() && paren > 0 {
                    match data[i] {
                        b'\\' => { i += 2; } // séquence d'échappement
                        b'(' => { paren += 1; i += 1; }
                        b')' => { paren -= 1; i += 1; }
                        _ => { i += 1; }
                    }
                }
            }
            b'<' if data.get(i + 1) == Some(&b'<') => {
                depth += 1;
                i += 2;
            }
            b'<' => {
                // Chaîne hex — ignorer jusqu'au prochain `>`
                i += 1;
                while i < data.len() && data[i] != b'>' {
                    i += 1;
                }
                if i < data.len() { i += 1; }
            }
            b'>' if data.get(i + 1) == Some(&b'>') => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
                i += 2;
            }
            b'%' => {
                // Commentaire — ignorer jusqu'à fin de ligne
                while i < data.len() && data[i] != b'\n' && data[i] != b'\r' {
                    i += 1;
                }
            }
            _ => { i += 1; }
        }
    }

    None
}

// --- Suppression du dictionnaire /Info ---

fn remove_info_object(data: &mut [u8], obj_num: usize) {
    let pattern = format!("{} 0 obj", obj_num);
    let obj_bytes = pattern.as_bytes();

    let Some(obj_pos) = find_bytes(data, obj_bytes) else { return };
    let search_start = obj_pos + obj_bytes.len();
    let search_end = (search_start + 1000).min(data.len());

    let Some(dict_open_rel) = find_bytes(&data[search_start..search_end], b"<<") else { return };
    let dict_open = search_start + dict_open_rel;

    let Some(dict_close) = find_dict_end(data, dict_open + 2) else { return };

    // Effacer uniquement le contenu entre << et >> (les délimiteurs restent)
    erase(data, dict_open + 2, dict_close);
}

// --- Suppression de la référence /Info dans le trailer ---

fn remove_info_from_trailer(data: &mut [u8]) {
    let Some(trailer_pos) = rfind_bytes(data, b"trailer") else { return };
    let search_end = (trailer_pos + 2000).min(data.len());

    let Some(info_rel) = find_bytes(&data[trailer_pos..search_end], b"/Info") else { return };
    let info_pos = trailer_pos + info_rel;

    // La référence a la forme "/Info N G R" — on efface jusqu'au " R" inclus
    let end_search = (info_pos + 50).min(data.len());
    let Some(r_rel) = find_bytes(&data[info_pos..end_search], b" R") else { return };
    let end_pos = info_pos + r_rel + 2;

    erase(data, info_pos, end_pos);
}

// --- Suppression des métadonnées XMP ---

fn remove_xmp_xpacket(data: &mut [u8]) {
    let start_pat = b"<?xpacket begin";
    let end_pat = b"<?xpacket end";
    let close = b"?>";

    let mut pos = 0;
    while let Some(start_rel) = find_bytes(&data[pos..], start_pat) {
        let abs_start = pos + start_rel;
        let after_start = abs_start + start_pat.len();

        let Some(end_tag_rel) = find_bytes(&data[after_start..], end_pat) else { break };
        let abs_end_tag = after_start + end_tag_rel + end_pat.len();

        let Some(close_rel) = find_bytes(&data[abs_end_tag..], close) else { break };
        let abs_end = abs_end_tag + close_rel + close.len();

        erase(data, abs_start, abs_end);
        pos = abs_start + 1;
    }
}

fn remove_xmpmeta(data: &mut [u8]) {
    let start_pat = b"<x:xmpmeta";
    let end_pat = b"</x:xmpmeta>";

    let mut pos = 0;
    while let Some(start_rel) = find_bytes(&data[pos..], start_pat) {
        let abs_start = pos + start_rel;

        let Some(end_rel) = find_bytes(&data[abs_start..], end_pat) else { break };
        let abs_end = abs_start + end_rel + end_pat.len();

        erase(data, abs_start, abs_end);
        pos = abs_start + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_pdf_invalid() {
        assert!(sanitize_pdf(b"NOT A PDF").is_err());
    }

    #[test]
    fn test_sanitize_pdf_minimal() {
        let pdf = b"%PDF-1.4\ntrailer\n<< >>\n%%EOF";
        assert!(sanitize_pdf(pdf).is_ok());
    }

    #[test]
    fn test_sanitize_pdf_no_trailer_passthrough() {
        // PDF 1.5+ XRef stream uniquement — non supporté
        let pdf = b"%PDF-1.5\nstartxref\n0\n%%EOF";
        assert!(matches!(sanitize_pdf(pdf), Err(Error::UnsupportedFormat(_))));
    }

    #[test]
    fn test_find_dict_end_simple() {
        let data = b"<</Author (John)>>";
        assert_eq!(find_dict_end(data, 2), Some(16));
    }

    #[test]
    fn test_find_dict_end_nested() {
        let data = b"<</Nested <</Key (val)>> /Other (x)>>";
        assert_eq!(find_dict_end(data, 2), Some(35));
    }

    #[test]
    fn test_find_dict_end_string_with_parens() {
        let data = b"<</Key (hello (world))>>";
        assert_eq!(find_dict_end(data, 2), Some(22));
    }

    #[test]
    fn test_remove_info_object() {
        let mut data = b"5 0 obj\n<</Author (John Doe) /Title (Test)>>\nendobj".to_vec();
        remove_info_object(&mut data, 5);
        assert!(!data.windows(6).any(|w| w == b"Author"));
        assert!(!data.windows(5).any(|w| w == b"Title"));
        // Les délimiteurs << et >> restent
        assert!(data.windows(2).any(|w| w == b"<<"));
        assert!(data.windows(2).any(|w| w == b">>"));
    }

    #[test]
    fn test_remove_info_from_trailer() {
        let mut data = b"trailer\n<< /Info 5 0 R /Root 1 0 R >>\n".to_vec();
        remove_info_from_trailer(&mut data);
        assert!(!data.windows(5).any(|w| w == b"/Info"));
        // /Root doit rester intact
        assert!(data.windows(5).any(|w| w == b"/Root"));
    }

    #[test]
    fn test_remove_xmpmeta() {
        let mut data = b"before<x:xmpmeta>secret data</x:xmpmeta>after".to_vec();
        remove_xmpmeta(&mut data);
        assert!(!data.windows(10).any(|w| w == b"<x:xmpmeta"));
        assert!(!data.windows(6).any(|w| w == b"secret"));
        // La taille ne change pas (effacement en place)
        assert_eq!(data.len(), 45);
    }

    #[test]
    fn test_remove_xmp_xpacket() {
        let mut data =
            b"A<?xpacket begin='utf8'?>content<?xpacket end='r'?>B".to_vec();
        remove_xmp_xpacket(&mut data);
        assert!(!data.windows(15).any(|w| w == b"<?xpacket begin"));
        assert!(!data.windows(7).any(|w| w == b"content"));
        assert_eq!(data.len(), 52);
    }

    #[test]
    fn test_verify_sanitization_clean() {
        assert!(verify_sanitization(b"%PDF-1.4\ntrailer\n<< >>\n%%EOF"));
    }

    #[test]
    fn test_verify_sanitization_dirty() {
        assert!(!verify_sanitization(b"<?xpacket begin='utf8'?>"));
        assert!(!verify_sanitization(b"<x:xmpmeta>data</x:xmpmeta>"));
    }

    #[test]
    fn test_erase_preserves_length() {
        let mut data = b"Hello World".to_vec();
        let original_len = data.len();
        erase(&mut data, 6, 11);
        assert_eq!(data.len(), original_len);
        assert_eq!(&data[..6], b"Hello ");
        assert_eq!(&data[6..], b"     ");
    }
}
