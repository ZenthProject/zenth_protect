//! Module MP3 - Parsing et sanitization des fichiers audio MP3
//!
//! # Format MP3
//!
//! Un fichier MP3 peut contenir plusieurs types de métadonnées :
//!
//! 1. **ID3v1** (128 bytes à la fin du fichier)
//!    - Format fixe : TAG + Title(30) + Artist(30) + Album(30) + Year(4) + Comment(30) + Genre(1)
//!    - Simple mais limité
//!
//! 2. **ID3v2** (au début du fichier)
//!    - Format variable avec frames
//!    - Header : "ID3" + version + flags + size
//!    - Frames : ID(4) + Size(4) + Flags(2) + Data
//!    - Peut contenir : titre, artiste, album, image, GPS, commentaires, etc.
//!
//! 3. **APE Tags** (à la fin, avant ID3v1)
//!    - Format alternatif utilisé par certains logiciels
//!
//! # Métadonnées dangereuses
//!
//! - TXXX : Champs personnalisés (peut contenir GPS, identifiants)
//! - PRIV : Données privées
//! - GEOB : Objets embarqués
//! - APIC : Images (peuvent contenir EXIF)
//! - COMM : Commentaires
//! - USLT : Paroles (peuvent identifier l'utilisateur)

pub mod metadata_mp3;
pub mod parser_mp3;
pub mod sanitize_mp3;
