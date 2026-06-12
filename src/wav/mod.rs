//! Module WAV - Parsing et sanitization des fichiers audio WAV
//!
//! # Format WAV (RIFF/WAVE)
//!
//! WAV utilise le conteneur RIFF (Resource Interchange File Format) :
//!
//! ```text
//! ┌────────────────────────────────────────────┐
//! │ RIFF Header                                │
//! │ ├─ "RIFF" (4 bytes)                       │
//! │ ├─ File size - 8 (4 bytes, little-endian) │
//! │ └─ "WAVE" (4 bytes)                       │
//! ├────────────────────────────────────────────┤
//! │ Chunks (sous-blocs)                        │
//! │ ├─ "fmt " : Format audio                  │
//! │ ├─ "data" : Données audio brutes          │
//! │ ├─ "LIST" : Métadonnées (INFO, etc.)      │ [DANGER]
//! │ ├─ "id3 " : Tag ID3 embarqué              │ [DANGER]
//! │ ├─ "bext" : Broadcast Extension           │ [DANGER]
//! │ └─ autres chunks optionnels               │
//! └────────────────────────────────────────────┘
//! ```
//!
//! # Métadonnées dangereuses
//!
//! - `LIST INFO` : Artiste, titre, copyright, commentaires, logiciel
//! - `id3 ` : Tag ID3v2 embarqué (comme MP3)
//! - `bext` : Broadcast Extension (origine, date, codeur)
//! - `cart` : Cart chunk (radio broadcast metadata)
//! - `cue ` : Points de repère (peut révéler des informations)
//! - `plst` : Playlist
//! - `smpl` : Sample info (peut contenir des commentaires)
//! - `inst` : Instrument info
//! - `_PMX` : XMP metadata (Adobe)

pub mod chunk_wav;
pub mod parser_wav;
pub mod sanitize_wav;
