/// Module MP4 - Parsing et sanitization des fichiers vidéo MP4/M4A/MOV
///
/// # Format MP4 (ISO Base Media File Format)
///
/// MP4 est basé sur les "atoms" (aussi appelés "boxes") :
///
/// ```text
/// ┌────────────────────────────────┐
/// │ Atom                           │
/// │ ├─ Size (4 bytes, big-endian) │
/// │ ├─ Type (4 bytes ASCII)        │
/// │ └─ Data (Size - 8 bytes)       │
/// └────────────────────────────────┘
/// ```
///
/// # Atoms principaux
///
/// - `ftyp` : File type (identifie le format)
/// - `moov` : Movie (contient les métadonnées et la structure)
///   - `mvhd` : Movie header
///   - `trak` : Track (audio/vidéo)
///   - `udta` : User data (métadonnées utilisateur)
///     - `meta` : Metadata (iTunes-style)
///       - `ilst` : Item list (tags)
/// - `mdat` : Media data (données audio/vidéo brutes)
/// - `free` : Free space
///
/// # Métadonnées dangereuses
///
/// - `udta` : Données utilisateur (auteur, titre, GPS, etc.)
/// - `meta` : Métadonnées iTunes (©ART, ©alb, ©day, etc.)
/// - `uuid` : Extensions propriétaires
/// - `XMP_` : Métadonnées XMP Adobe
/// - `©xyz` : Coordonnées GPS (iPhone)

pub mod atom_mp4;
pub mod parser_mp4;
pub mod sanitize_mp4;
