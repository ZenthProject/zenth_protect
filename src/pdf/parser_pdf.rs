pub fn is_pdf_signature(data: &[u8]) -> bool {
    data.len() >= 4 &&
    data[0] == b'%' &&
    data[1] == b'P' &&
    data[2] == b'D' &&
    data[3] == b'F'
}

pub fn find_pdf_version(data: &[u8]) -> Option<String> {
    if data.len() < 8 {
        return None;
    }

    if !is_pdf_signature(data) {
        return None;
    }

    let mut version_bytes = Vec::new();

    for i in 5..data.len() {
        let byte = data[i];

        if byte == b'\n' || byte == b'\r' {
            break;
        }

        version_bytes.push(byte);

        if version_bytes.len() > 10 {
            break;
        }
    }

    String::from_utf8(version_bytes).ok()
}

pub fn find_info_object_number(data: &[u8]) -> Option<usize> {
    let content = String::from_utf8_lossy(data);

    let trailer_pos = content.rfind("trailer")?;

    let trailer_section = &content[trailer_pos..];
    let search_section = if trailer_section.len() > 2000 {
        &trailer_section[..2000]
    } else {
        trailer_section
    };

    let info_pos = search_section.find("/Info")?;

    let after_info = &search_section[info_pos + 5..];

    let trimmed = after_info.trim_start();

    let mut number_str = String::new();
    for ch in trimmed.chars() {
        if ch.is_ascii_digit() {
            number_str.push(ch);
        } else if !number_str.is_empty() {
            break;
        }
    }

    number_str.parse::<usize>().ok()
}

pub fn extract_info_dict(data: &[u8], obj_num: usize) -> Option<super::metadata_pdf::PdfMetadata> {
    let content = String::from_utf8_lossy(data);

    let obj_pattern = format!("{} 0 obj", obj_num);
    let obj_pos = content.find(&obj_pattern)?;

    let obj_section = &content[obj_pos..];
    let search_section = if obj_section.len() > 10000 {
        &obj_section[..10000]
    } else {
        obj_section
    };

    let dict_start = search_section.find("<<")?;
    let dict_end = search_section.find(">>")?;

    if dict_end <= dict_start {
        return None;
    }

    let dict_content = &search_section[dict_start + 2..dict_end];

    let mut metadata = super::metadata_pdf::PdfMetadata::new();

    metadata.author = extract_pdf_string(dict_content, "/Author");
    metadata.creator = extract_pdf_string(dict_content, "/Creator");
    metadata.producer = extract_pdf_string(dict_content, "/Producer");
    metadata.creation_date = extract_pdf_string(dict_content, "/CreationDate");
    metadata.mod_date = extract_pdf_string(dict_content, "/ModDate");
    metadata.title = extract_pdf_string(dict_content, "/Title");
    metadata.subject = extract_pdf_string(dict_content, "/Subject");
    metadata.keywords = extract_pdf_string(dict_content, "/Keywords");

    let mut pos = 0;
    while let Some(slash_pos) = dict_content[pos..].find('/') {
        let abs_pos = pos + slash_pos;

        let after_slash = &dict_content[abs_pos + 1..];
        let key_end = after_slash
            .find(|c: char| c.is_whitespace() || c == '(')
            .unwrap_or(after_slash.len());

        let key = &after_slash[..key_end];

        let is_standard = matches!(
            key,
            "Author" | "Creator" | "Producer" | "CreationDate"
            | "ModDate" | "Title" | "Subject" | "Keywords"
        );

        if !is_standard && !key.is_empty() {
            if let Some(value) = extract_pdf_string(dict_content, &format!("/{}", key)) {
                metadata.custom.push((key.to_string(), value));
            }
        }

        pos = abs_pos + 1;
    }

    Some(metadata)
}

fn extract_pdf_string(content: &str, key: &str) -> Option<String> {
    let key_pos = content.find(key)?;

    let after_key = &content[key_pos + key.len()..];

    let open_paren = after_key.find('(')?;

    let value_start = &after_key[open_paren + 1..];

    let mut paren_count = 1;
    let mut close_pos = None;

    for (i, ch) in value_start.chars().enumerate() {
        match ch {
            '(' => paren_count += 1,
            ')' => {
                paren_count -= 1;
                if paren_count == 0 {
                    close_pos = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }

    let close_paren = close_pos?;

    let value = &value_start[..close_paren];

    Some(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_pdf_signature() {
        assert!(is_pdf_signature(b"%PDF-1.4\n"));
        assert!(is_pdf_signature(b"%PDF-2.0\r\n"));
        assert!(!is_pdf_signature(b"NOTAPDF!"));
        assert!(!is_pdf_signature(b"%PD"));
    }

    #[test]
    fn test_find_pdf_version() {
        assert_eq!(find_pdf_version(b"%PDF-1.4\n"), Some("1.4".to_string()));
        assert_eq!(find_pdf_version(b"%PDF-2.0\r\n"), Some("2.0".to_string()));
        assert_eq!(find_pdf_version(b"NOTAPDF"), None);
    }

    #[test]
    fn test_find_info_object_number() {
        let pdf = b"%PDF-1.4\ntrailer\n<< /Info 5 0 R >>\n";
        assert_eq!(find_info_object_number(pdf), Some(5));
    }

    #[test]
    fn test_find_info_object_number_not_found() {
        let pdf = b"%PDF-1.4\ntrailer\n<< >>\n";
        assert_eq!(find_info_object_number(pdf), None);
    }

    #[test]
    fn test_extract_pdf_string() {
        let content = "/Author (John Doe) /Title (Test)";
        assert_eq!(extract_pdf_string(content, "/Author"), Some("John Doe".to_string()));
        assert_eq!(extract_pdf_string(content, "/Title"), Some("Test".to_string()));
        assert_eq!(extract_pdf_string(content, "/Missing"), None);
    }

    #[test]
    fn test_extract_pdf_string_nested_parens() {
        let content = "/Title (Report (draft))";
        assert_eq!(extract_pdf_string(content, "/Title"), Some("Report (draft)".to_string()));
    }
}
