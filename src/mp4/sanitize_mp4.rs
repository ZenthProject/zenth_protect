use crate::error::{Error, Result};
use super::parser_mp4::{is_mp4_signature, list_top_level_atoms};
use super::atom_mp4::Mp4Atom;

fn read_u32_be(data: &[u8], offset: usize) -> Option<u32> {
    if offset.checked_add(4)? > data.len() {
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
    if offset.checked_add(8)? > data.len() {
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

fn is_allowed_top_level(atom_type: &[u8; 4]) -> bool {
    matches!(atom_type, b"ftyp" | b"moov" | b"mdat")
}

fn is_allowed_in_moov(atom_type: &[u8; 4]) -> bool {
    matches!(atom_type, b"mvhd" | b"trak" | b"iods")
}

fn is_allowed_in_trak(atom_type: &[u8; 4]) -> bool {
    matches!(atom_type, b"tkhd" | b"mdia" | b"edts" | b"tref")
}

fn is_allowed_in_mdia(atom_type: &[u8; 4]) -> bool {
    matches!(atom_type, b"mdhd" | b"hdlr" | b"minf")
}

fn is_allowed_in_minf(atom_type: &[u8; 4]) -> bool {
    matches!(atom_type, b"vmhd" | b"smhd" | b"hmhd" | b"nmhd" | b"dinf" | b"stbl")
}

fn is_allowed_in_stbl(atom_type: &[u8; 4]) -> bool {
    matches!(
        atom_type,
        b"stsd" | b"stts" | b"stsc" | b"stsz" | b"stz2" | b"stco" | b"co64" | b"stss" | b"ctts" | b"sdtp" | b"sbgp" | b"sgpd"
    )
}

fn is_allowed_in_dinf(atom_type: &[u8; 4]) -> bool {
    matches!(atom_type, b"dref")
}

fn is_allowed_in_edts(atom_type: &[u8; 4]) -> bool {
    matches!(atom_type, b"elst")
}

fn sanitize_container(
    container_data: &[u8],
    container_type: &[u8; 4],
    is_allowed: fn(&[u8; 4]) -> bool,
) -> Vec<u8> {
    let mut result = Vec::new();
    result.extend_from_slice(&[0, 0, 0, 0]);
    result.extend_from_slice(container_type);

    let mut offset = 8;

    while offset + 8 <= container_data.len() {
        let child_size = match read_u32_be(container_data, offset) {
            Some(size) => size as usize,
            None => break,
        };

        let child_type: [u8; 4] = match container_data.get(offset + 4..offset + 8) {
            Some(slice) => match slice.try_into() {
                Ok(arr) => arr,
                Err(_) => break,
            },
            None => break,
        };

        if child_size < 8 {
            break;
        }

        let child_end = match offset.checked_add(child_size) {
            Some(end) if end <= container_data.len() => end,
            _ => break,
        };

        let child_data = &container_data[offset..child_end];

        if is_allowed(&child_type) {
            let sanitized = match &child_type {
                b"trak" => sanitize_container(child_data, b"trak", is_allowed_in_trak),
                b"mdia" => sanitize_container(child_data, b"mdia", is_allowed_in_mdia),
                b"minf" => sanitize_container(child_data, b"minf", is_allowed_in_minf),
                b"stbl" => sanitize_container(child_data, b"stbl", is_allowed_in_stbl),
                b"dinf" => sanitize_container(child_data, b"dinf", is_allowed_in_dinf),
                b"edts" => sanitize_container(child_data, b"edts", is_allowed_in_edts),
                _ => child_data.to_vec(),
            };
            result.extend(sanitized);
        }

        offset = child_end;
    }

    let new_size = result.len() as u32;
    result[0] = (new_size >> 24) as u8;
    result[1] = (new_size >> 16) as u8;
    result[2] = (new_size >> 8) as u8;
    result[3] = new_size as u8;

    result
}

fn sanitize_moov_atom(data: &[u8], moov: &Mp4Atom) -> Vec<u8> {
    let moov_start = moov.offset;

    let moov_end = match moov_start.checked_add(moov.size as usize) {
        Some(end) if end <= data.len() => end,
        _ => {
            let mut empty_moov = Vec::new();
            empty_moov.extend_from_slice(&8u32.to_be_bytes());
            empty_moov.extend_from_slice(b"moov");
            return empty_moov;
        }
    };

    let moov_data = &data[moov_start..moov_end];
    sanitize_container(moov_data, b"moov", is_allowed_in_moov)
}

fn update_stco_offsets(data: &mut [u8], offset_delta: i64) {
    let mut pos = 0;

    while pos + 8 <= data.len() {
        let atom_size = match read_u32_be(data, pos) {
            Some(size) => size as usize,
            None => break,
        };

        if atom_size < 8 {
            break;
        }

        let atom_end = match pos.checked_add(atom_size) {
            Some(end) if end <= data.len() => end,
            _ => break,
        };

        let atom_type: [u8; 4] = match data.get(pos + 4..pos + 8) {
            Some(slice) => match slice.try_into() {
                Ok(arr) => arr,
                Err(_) => break,
            },
            None => break,
        };

        if atom_type == *b"stco" && atom_size > 16 {
            let entry_count = match read_u32_be(data, pos + 12) {
                Some(count) => count as usize,
                None => continue,
            };
            for i in 0..entry_count {
                let entry_offset = match (pos + 16).checked_add(i.saturating_mul(4)) {
                    Some(offset) if offset + 4 <= data.len() => offset,
                    _ => break,
                };
                if let Some(old) = read_u32_be(data, entry_offset) {
                    let new = ((old as i64) + offset_delta) as u32;
                    data[entry_offset] = (new >> 24) as u8;
                    data[entry_offset + 1] = (new >> 16) as u8;
                    data[entry_offset + 2] = (new >> 8) as u8;
                    data[entry_offset + 3] = new as u8;
                }
            }
        } else if atom_type == *b"co64" && atom_size > 16 {
            let entry_count = match read_u32_be(data, pos + 12) {
                Some(count) => count as usize,
                None => continue,
            };
            for i in 0..entry_count {
                let entry_offset = match (pos + 16).checked_add(i.saturating_mul(8)) {
                    Some(offset) if offset + 8 <= data.len() => offset,
                    _ => break,
                };
                if let Some(old) = read_u64_be(data, entry_offset) {
                    let new = ((old as i64) + offset_delta) as u64;
                    data[entry_offset] = (new >> 56) as u8;
                    data[entry_offset + 1] = (new >> 48) as u8;
                    data[entry_offset + 2] = (new >> 40) as u8;
                    data[entry_offset + 3] = (new >> 32) as u8;
                    data[entry_offset + 4] = (new >> 24) as u8;
                    data[entry_offset + 5] = (new >> 16) as u8;
                    data[entry_offset + 6] = (new >> 8) as u8;
                    data[entry_offset + 7] = new as u8;
                }
            }
        }

        let is_container = matches!(&atom_type, b"moov" | b"trak" | b"mdia" | b"minf" | b"stbl");

        if is_container && atom_size > 8 {
            update_stco_offsets(&mut data[pos + 8..atom_end], offset_delta);
        }

        pos = atom_end;
    }
}

pub fn sanitize_mp4(data: &[u8]) -> Result<Vec<u8>> {
    if !is_mp4_signature(data) {
        return Err(Error::InvalidSignature("MP4"));
    }

    let top_atoms = list_top_level_atoms(data);
    let mdat_atom = top_atoms.iter().find(|a| a.atom_type == *b"mdat");
    let original_mdat_offset = mdat_atom.map(|a| a.offset).unwrap_or(0);

    let mut result = Vec::new();
    let mut removed_before_mdat = 0usize;

    for atom in &top_atoms {
        let atom_end = match atom.offset.checked_add(atom.size as usize) {
            Some(end) if end <= data.len() => end,
            _ => continue,
        };

        if is_allowed_top_level(&atom.atom_type) {
            match &atom.atom_type {
                b"moov" => {
                    let old_size = atom.size as usize;
                    let sanitized_moov = sanitize_moov_atom(data, atom);
                    let new_size = sanitized_moov.len();

                    if atom.offset < original_mdat_offset && old_size > new_size {
                        removed_before_mdat += old_size - new_size;
                    }

                    result.extend(sanitized_moov);
                }
                _ => {
                    let atom_data = &data[atom.offset..atom_end];
                    result.extend_from_slice(atom_data);
                }
            }
        } else if atom.offset < original_mdat_offset {
            removed_before_mdat += atom.size as usize;
        }
    }

    if removed_before_mdat > 0 {
        update_stco_offsets(&mut result, -(removed_before_mdat as i64));
    }

    Ok(result)
}

pub fn verify_sanitization(data: &[u8]) -> bool {
    if !is_mp4_signature(data) {
        return false;
    }

    let atoms = list_top_level_atoms(data);

    for atom in &atoms {
        if !is_allowed_top_level(&atom.atom_type) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u32_be_safe() {
        let data = [0x00, 0x00, 0x00, 0x10];
        assert_eq!(read_u32_be(&data, 0), Some(16));
        assert_eq!(read_u32_be(&data, 1), None);
    }

    #[test]
    fn test_read_u64_be_safe() {
        let data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20];
        assert_eq!(read_u64_be(&data, 0), Some(32));
        assert_eq!(read_u64_be(&data, 1), None);
    }

    #[test]
    fn test_is_allowed_top_level() {
        assert!(is_allowed_top_level(b"ftyp"));
        assert!(is_allowed_top_level(b"moov"));
        assert!(is_allowed_top_level(b"mdat"));
        assert!(!is_allowed_top_level(b"udta"));
    }

    #[test]
    fn test_is_allowed_in_moov() {
        assert!(is_allowed_in_moov(b"mvhd"));
        assert!(is_allowed_in_moov(b"trak"));
        assert!(!is_allowed_in_moov(b"udta"));
    }

    #[test]
    fn test_is_allowed_in_stbl() {
        assert!(is_allowed_in_stbl(b"stsd"));
        assert!(is_allowed_in_stbl(b"stco"));
        assert!(!is_allowed_in_stbl(b"meta"));
    }

    #[test]
    fn test_sanitize_mp4_invalid() {
        let invalid = b"NOT AN MP4 FILE";
        assert!(sanitize_mp4(invalid).is_err());
    }

    #[test]
    fn test_sanitize_container_empty() {
        let mut container = Vec::new();
        container.extend_from_slice(&8u32.to_be_bytes());
        container.extend_from_slice(b"moov");

        let result = sanitize_container(&container, b"moov", is_allowed_in_moov);
        assert_eq!(result.len(), 8);
    }
}
