use std::fmt;
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    InvalidSignature(&'static str),
    TruncatedFile,
    InvalidChunk(&'static str),
    InvalidHeader(&'static str),
    CorruptedData,
    BufferOverflow,
    NoAudioData,
    WriteError,
    
    // Nouvelles variantes pour le parser
    UnsupportedFormat(&'static str),
    NoVideoData,
    InvalidFileSize(usize),
    InvalidMetadata(&'static str),
    Io(String),
    FileTooSmall { expected: usize, actual: usize },
    FileTooLarge { max: usize, actual: usize },
    ExtensionMismatch { extension: String, detected: String },
    UnsupportedVersion { format: &'static str, version: String },
    MissingData(&'static str),
    InvalidOffset { offset: usize, file_size: usize },
    ParseError(&'static str),
    SanitizationFailed(&'static str),
    SuspiciousData(&'static str),
    InvalidEncoding(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidSignature(fmt) => write!(f, "Invalid {} signature", fmt),
            Error::TruncatedFile => write!(f, "Truncated file"),
            Error::InvalidChunk(msg) => write!(f, "Invalid chunk: {}", msg),
            Error::InvalidHeader(msg) => write!(f, "Invalid header: {}", msg),
            Error::CorruptedData => write!(f, "Corrupted data"),
            Error::BufferOverflow => write!(f, "Buffer overflow"),
            Error::NoAudioData => write!(f, "No audio data found"),
            Error::WriteError => write!(f, "Write error"),
            
            // Nouveaux messages
            Error::UnsupportedFormat(msg) => write!(f, "Unsupported file format: {}", msg),
            Error::NoVideoData => write!(f, "No video data found in file"),
            Error::InvalidFileSize(size) => write!(f, "Invalid file size: {} bytes", size),
            Error::InvalidMetadata(msg) => write!(f, "Invalid metadata: {}", msg),
            Error::Io(msg) => write!(f, "I/O error: {}", msg),
            Error::FileTooSmall { expected, actual } => {
                write!(f, "File too small: expected at least {} bytes, got {}", expected, actual)
            }
            Error::FileTooLarge { max, actual } => {
                write!(f, "File too large: maximum {} bytes, got {}", max, actual)
            }
            Error::ExtensionMismatch { extension, detected } => {
                write!(
                    f,
                    "Extension mismatch: file has extension '{}' but content is '{}'",
                    extension, detected
                )
            }
            Error::UnsupportedVersion { format, version } => {
                write!(f, "Unsupported {} version: {}", format, version)
            }
            Error::MissingData(msg) => write!(f, "Missing required data: {}", msg),
            Error::InvalidOffset { offset, file_size } => {
                write!(f, "Invalid offset: {} exceeds file size {}", offset, file_size)
            }
            Error::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Error::SanitizationFailed(msg) => write!(f, "Sanitization failed: {}", msg),
            Error::SuspiciousData(msg) => write!(f, "Suspicious data detected: {}", msg),
            Error::InvalidEncoding(msg) => write!(f, "Invalid encoding: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

// Conversion depuis io::Error
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err.to_string())
    }
}

// Méthodes utilitaires
impl Error {
    /// Crée une erreur pour un fichier trop petit
    pub fn file_too_small(expected: usize, actual: usize) -> Self {
        Error::FileTooSmall { expected, actual }
    }

    /// Crée une erreur pour un fichier trop grand
    pub fn file_too_large(max: usize, actual: usize) -> Self {
        Error::FileTooLarge { max, actual }
    }

    /// Crée une erreur pour une extension qui ne correspond pas
    pub fn extension_mismatch(extension: impl Into<String>, detected: impl Into<String>) -> Self {
        Error::ExtensionMismatch {
            extension: extension.into(),
            detected: detected.into(),
        }
    }

    /// Crée une erreur pour une version non supportée
    pub fn unsupported_version(format: &'static str, version: impl Into<String>) -> Self {
        Error::UnsupportedVersion {
            format,
            version: version.into(),
        }
    }

    /// Crée une erreur pour un offset invalide
    pub fn invalid_offset(offset: usize, file_size: usize) -> Self {
        Error::InvalidOffset { offset, file_size }
    }

    /// Vérifie si le buffer a une taille minimale
    pub fn check_min_size(data: &[u8], min_size: usize) -> Result<()> {
        if data.len() < min_size {
            Err(Error::file_too_small(min_size, data.len()))
        } else {
            Ok(())
        }
    }

    /// Vérifie si un offset est valide
    pub fn check_offset(offset: usize, file_size: usize) -> Result<()> {
        if offset > file_size {
            Err(Error::invalid_offset(offset, file_size))
        } else {
            Ok(())
        }
    }

    /// Vérifie si une plage est valide
    pub fn check_range(start: usize, end: usize, file_size: usize) -> Result<()> {
        if start > end {
            Err(Error::ParseError("Start offset greater than end offset"))
        } else if end > file_size {
            Err(Error::invalid_offset(end, file_size))
        } else {
            Ok(())
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::InvalidSignature("MP3");
        assert_eq!(err.to_string(), "Invalid MP3 signature");

        let err = Error::NoAudioData;
        assert_eq!(err.to_string(), "No audio data found");

        let err = Error::TruncatedFile;
        assert_eq!(err.to_string(), "Truncated file");
    }

    #[test]
    fn test_file_too_small() {
        let err = Error::file_too_small(100, 50);
        assert!(err.to_string().contains("expected at least 100"));
        assert!(err.to_string().contains("got 50"));
    }

    #[test]
    fn test_extension_mismatch() {
        let err = Error::extension_mismatch("mp3", "jpeg");
        assert!(err.to_string().contains("mp3"));
        assert!(err.to_string().contains("jpeg"));
    }

    #[test]
    fn test_check_min_size() {
        let data = vec![0u8; 10];
        assert!(Error::check_min_size(&data, 5).is_ok());
        assert!(Error::check_min_size(&data, 20).is_err());
    }

    #[test]
    fn test_check_offset() {
        assert!(Error::check_offset(50, 100).is_ok());
        assert!(Error::check_offset(150, 100).is_err());
    }

    #[test]
    fn test_check_range() {
        assert!(Error::check_range(10, 50, 100).is_ok());
        assert!(Error::check_range(50, 10, 100).is_err());
        assert!(Error::check_range(10, 150, 100).is_err());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_error_equality() {
        let err1 = Error::InvalidSignature("MP3");
        let err2 = Error::InvalidSignature("MP3");
        let err3 = Error::InvalidSignature("JPEG");
        
        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_corrupted_data() {
        let err = Error::CorruptedData;
        assert_eq!(err.to_string(), "Corrupted data");
    }

    #[test]
    fn test_buffer_overflow() {
        let err = Error::BufferOverflow;
        assert_eq!(err.to_string(), "Buffer overflow");
    }

    #[test]
    fn test_unsupported_version() {
        let err = Error::unsupported_version("PDF", "2.0");
        assert!(err.to_string().contains("PDF"));
        assert!(err.to_string().contains("2.0"));
    }
}