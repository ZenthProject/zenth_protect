use super::atom_mp4::{Mp4Atom, Mp4Metadata};

pub fn is_mp4_signature(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }

    let atom_type = &data[4..8];

    if atom_type == b"ftyp" {
        return true;
    }

    if atom_type == b"moov" || atom_type == b"mdat" || atom_type == b"wide" || atom_type == b"free" {
        return true;
    }

    false
}

fn read_u32_be(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 4 > data.len() {
        return None;
    }
    Some(
        ((data[offset] as u32) << 24)
            | ((data[offset + 1] as u32) << 16)
            | ((data[offset + 2] as u32) << 8)
            | (data[offset + 3] as u32),
    )
}

fn read_u64_be(data: &[u8], offset: usize) -> Option<u64> {
    if offset + 8 > data.len() {
        return None;
    }
    Some(
        ((data[offset] as u64) << 56)
            | ((data[offset + 1] as u64) << 48)
            | ((data[offset + 2] as u64) << 40)
            | ((data[offset + 3] as u64) << 32)
            | ((data[offset + 4] as u64) << 24)
            | ((data[offset + 5] as u64) << 16)
            | ((data[offset + 6] as u64) << 8)
            | (data[offset + 7] as u64),
    )
}

pub fn read_atom(data: &[u8], offset: usize) -> Option<Mp4Atom> {
    if offset + 8 > data.len() {
        return None;
    }

    let size32 = read_u32_be(data, offset)?;

    let mut atom_type = [0u8; 4];
    atom_type.copy_from_slice(&data[offset + 4..offset + 8]);

    let (size, header_size) = if size32 == 1 {
        let size64 = read_u64_be(data, offset + 8)?;
        (size64, 16)
    } else if size32 == 0 {
        ((data.len() - offset) as u64, 8)
    } else {
        (size32 as u64, 8)
    };

    if offset as u64 + size > data.len() as u64 {
        let available_size = (data.len() - offset) as u64;
        return Some(Mp4Atom::new(atom_type, offset, available_size, header_size));
    }

    Some(Mp4Atom::new(atom_type, offset, size, header_size))
}

pub fn list_top_level_atoms(data: &[u8]) -> Vec<Mp4Atom> {
    let mut atoms = Vec::new();
    let mut offset = 0;

    while offset < data.len() {
        if let Some(atom) = read_atom(data, offset) {
            let atom_size = atom.size as usize;
            atoms.push(atom);

            if atom_size == 0 {
                break;
            }
            offset += atom_size;
        } else {
            break;
        }
    }

    atoms
}

pub fn list_all_atoms(data: &[u8]) -> Vec<Mp4Atom> {
    let mut all_atoms = Vec::new();
    list_atoms_recursive(data, 0, data.len(), &mut all_atoms, 0);
    all_atoms
}

fn list_atoms_recursive(
    data: &[u8],
    start: usize,
    end: usize,
    atoms: &mut Vec<Mp4Atom>,
    depth: usize,
) {
    if depth > 20 {
        return;
    }

    let mut offset = start;

    while offset + 8 <= end {
        if let Some(atom) = read_atom(data, offset) {
            let atom_size = atom.size as usize;
            let is_container = atom.is_container();

            atoms.push(atom.clone());

            if is_container && atom_size > atom.header_size {
                let child_start = offset + atom.header_size;
                let child_end = offset + atom_size;

                let actual_start = if atom.atom_type == *b"meta" {
                    child_start + 4
                } else {
                    child_start
                };

                if actual_start < child_end {
                    list_atoms_recursive(data, actual_start, child_end, atoms, depth + 1);
                }
            }

            if atom_size == 0 {
                break;
            }
            offset += atom_size;
        } else {
            break;
        }
    }
}

fn extract_itunes_string(data: &[u8], atom_offset: usize, atom_size: usize) -> Option<String> {
    let data_start = atom_offset + 8;

    if data_start + 16 > atom_offset + atom_size || data_start + 16 > data.len() {
        return None;
    }

    if &data[data_start + 4..data_start + 8] != b"data" {
        return None;
    }

    let string_start = data_start + 16;
    let data_atom_size = read_u32_be(data, data_start)? as usize;
    let string_end = std::cmp::min(data_start + data_atom_size, data.len());

    if string_start >= string_end {
        return None;
    }

    let string_bytes = &data[string_start..string_end];

    String::from_utf8(string_bytes.to_vec())
        .ok()
        .map(|s| s.trim_end_matches('\0').to_string())
        .filter(|s| !s.is_empty())
}

pub fn extract_metadata(data: &[u8]) -> Mp4Metadata {
    let mut metadata = Mp4Metadata::new();
    let atoms = list_all_atoms(data);

    for atom in &atoms {
        if atom.is_metadata() {
            let type_str = atom.type_string();
            if !metadata.metadata_atoms.contains(&type_str) {
                metadata.metadata_atoms.push(type_str.clone());
            }
            metadata.total_metadata_size += atom.size;
        }

        let atom_size = atom.size as usize;

        match &atom.atom_type {
            b"\xa9nam" => {
                metadata.title = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"\xa9ART" => {
                metadata.artist = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"\xa9alb" => {
                metadata.album = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"\xa9day" => {
                metadata.year = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"\xa9gen" | b"gnre" => {
                metadata.genre = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"\xa9cmt" => {
                metadata.comment = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"\xa9wrt" => {
                metadata.composer = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"\xa9too" => {
                metadata.encoder = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"\xa9xyz" => {
                metadata.gps_coordinates = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"cprt" => {
                metadata.copyright = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"desc" => {
                metadata.description = extract_itunes_string(data, atom.offset, atom_size);
            }
            b"covr" => {
                metadata.has_cover_art = true;
            }
            _ => {}
        }
    }

    metadata
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mp4_signature_ftyp() {
        let mut data = vec![0u8; 12];
        data[4..8].copy_from_slice(b"ftyp");
        assert!(is_mp4_signature(&data));
    }

    #[test]
    fn test_is_mp4_signature_moov() {
        let mut data = vec![0u8; 12];
        data[4..8].copy_from_slice(b"moov");
        assert!(is_mp4_signature(&data));
    }

    #[test]
    fn test_is_mp4_signature_invalid() {
        assert!(!is_mp4_signature(b"NOTMP4!!"));
        assert!(!is_mp4_signature(b""));
        assert!(!is_mp4_signature(&[0u8; 5]));
    }

    #[test]
    fn test_read_u32_be() {
        let data = [0x00, 0x00, 0x00, 0x10];
        assert_eq!(read_u32_be(&data, 0), Some(16));
    }

    #[test]
    fn test_read_u32_be_out_of_bounds() {
        let data = [0x00, 0x00];
        assert_eq!(read_u32_be(&data, 0), None);
    }

    #[test]
    fn test_read_u64_be() {
        let data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20];
        assert_eq!(read_u64_be(&data, 0), Some(32));
    }

    #[test]
    fn test_read_atom() {
        let mut data = vec![0u8; 16];
        data[0..4].copy_from_slice(&16u32.to_be_bytes());
        data[4..8].copy_from_slice(b"ftyp");

        let atom = read_atom(&data, 0).unwrap();
        assert_eq!(atom.size, 16);
        assert_eq!(&atom.atom_type, b"ftyp");
    }

    #[test]
    fn test_list_top_level_atoms() {
        let mut data = vec![0u8; 32];
        data[0..4].copy_from_slice(&16u32.to_be_bytes());
        data[4..8].copy_from_slice(b"ftyp");
        data[16..20].copy_from_slice(&16u32.to_be_bytes());
        data[20..24].copy_from_slice(b"moov");

        let atoms = list_top_level_atoms(&data);
        assert_eq!(atoms.len(), 2);
    }
}
