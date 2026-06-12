pub(crate) mod cursor;
pub mod error;

pub use error::{Error, Result};

// Modules de format de fichiers
pub mod png;
pub mod jpeg;
pub mod pdf;
pub mod mp3;
pub mod mp4;
pub mod wav;

// Réexporter les fonctions de sanitisation principales
pub use png::sanitize_png::sanitize_png;
pub use jpeg::sanitize_jpeg::sanitize_jpeg;
pub use pdf::sanitize_pdf::sanitize_pdf;
pub use mp3::sanitize_mp3::sanitize_mp3;
pub use mp4::sanitize_mp4::sanitize_mp4;
pub use wav::sanitize_wav::sanitize_wav;

pub fn detect_file_type(data: &[u8]) -> &'static str {
    if data.len() < 12 {
        return "UNKNOWN";
    }

    if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        return "PNG";
    }

    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return "JPEG";
    }

    if data[0] == b'%' && data[1] == b'P' && data[2] == b'D' && data[3] == b'F' {
        return "PDF";
    }

    if (data[0] == b'I' && data[1] == b'D' && data[2] == b'3')
        || (data[0] == 0xFF && (data[1] & 0xE0) == 0xE0)
    {
        return "MP3";
    }

    if &data[4..8] == b"ftyp" || &data[4..8] == b"moov" || &data[4..8] == b"mdat" {
        return "MP4";
    }

    if &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" {
        return "WAV";
    }

    "UNKNOWN"
}
