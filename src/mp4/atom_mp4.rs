//! Structure d'un atom MP4
//!
//! Un atom est l'unité de base du format MP4.
//! Chaque atom a une taille et un type (4 caractères ASCII).

/// Représente un atom MP4
#[derive(Debug, Clone)]
pub struct Mp4Atom {
    /// Type de l'atom (4 caractères ASCII)
    pub atom_type: [u8; 4],

    /// Position dans le fichier (offset en bytes)
    pub offset: usize,

    /// Taille totale de l'atom (incluant header)
    pub size: u64,

    /// Taille du header (8 bytes normalement, 16 si extended)
    pub header_size: usize,
}

impl Mp4Atom {
    /// Crée un nouvel atom
    pub fn new(atom_type: [u8; 4], offset: usize, size: u64, header_size: usize) -> Self {
        Self {
            atom_type,
            offset,
            size,
            header_size,
        }
    }

    /// Retourne le type comme une chaîne de caractères
    pub fn type_string(&self) -> String {
        self.atom_type
            .iter()
            .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '?' })
            .collect()
    }

    /// Position des données (après le header)
    pub fn data_offset(&self) -> usize {
        self.offset + self.header_size
    }

    /// Taille des données (sans le header)
    pub fn data_size(&self) -> u64 {
        self.size.saturating_sub(self.header_size as u64)
    }

    /// Vérifie si c'est un atom conteneur (peut avoir des sous-atoms)
    pub fn is_container(&self) -> bool {
        matches!(
            &self.atom_type,
            b"moov" | b"trak" | b"mdia" | b"minf" | b"stbl" | b"dinf"
            | b"udta" | b"meta" | b"ilst" | b"edts" | b"clip" | b"matt"
            | b"tref" | b"gmhd"
        )
    }

    /// Vérifie si c'est un atom de métadonnées à supprimer
    pub fn is_metadata(&self) -> bool {
        // Atoms de métadonnées directs
        if matches!(
            &self.atom_type,
            b"udta" | b"meta" | b"uuid" | b"XMP_" | b"Xtra"
        ) {
            return true;
        }

        // Atoms iTunes (commencent par © ou sont des noms connus)
        let type_str = self.type_string();
        if type_str.starts_with('©') || type_str.starts_with('\u{a9}') {
            return true;
        }

        // Atoms iTunes sans ©
        matches!(
            &self.atom_type,
            b"aART" | b"akID" | b"apID" | b"atID" | b"cmID" | b"cnID"
            | b"covr" | b"cpil" | b"cprt" | b"desc" | b"disk" | b"egid"
            | b"geID" | b"gnre" | b"hdvd" | b"keyw" | b"ldes" | b"pcst"
            | b"pgap" | b"plID" | b"purd" | b"purl" | b"rtng" | b"sfID"
            | b"shwm" | b"soaa" | b"soal" | b"soar" | b"soco" | b"sonm"
            | b"sosn" | b"stik" | b"tmpo" | b"trkn" | b"tven" | b"tves"
            | b"tvnn" | b"tvsh" | b"tvsn"
        )
    }

    /// Vérifie si c'est un atom essentiel à garder
    pub fn is_essential(&self) -> bool {
        matches!(
            &self.atom_type,
            b"ftyp" | b"moov" | b"mvhd" | b"trak" | b"tkhd" | b"mdia"
            | b"mdhd" | b"hdlr" | b"minf" | b"vmhd" | b"smhd" | b"dinf"
            | b"dref" | b"stbl" | b"stsd" | b"stts" | b"stsc" | b"stsz"
            | b"stco" | b"co64" | b"ctts" | b"stss" | b"mdat" | b"free"
            | b"skip" | b"edts" | b"elst"
        )
    }

    /// Retourne une description humaine de l'atom
    pub fn description(&self) -> &'static str {
        match &self.atom_type {
            b"ftyp" => "File type",
            b"moov" => "Movie (metadata container)",
            b"mvhd" => "Movie header",
            b"trak" => "Track",
            b"tkhd" => "Track header",
            b"mdia" => "Media",
            b"mdhd" => "Media header",
            b"hdlr" => "Handler reference",
            b"minf" => "Media information",
            b"vmhd" => "Video media header",
            b"smhd" => "Sound media header",
            b"dinf" => "Data information",
            b"dref" => "Data reference",
            b"stbl" => "Sample table",
            b"stsd" => "Sample description",
            b"stts" => "Time-to-sample",
            b"stsc" => "Sample-to-chunk",
            b"stsz" => "Sample sizes",
            b"stco" => "Chunk offsets (32-bit)",
            b"co64" => "Chunk offsets (64-bit)",
            b"ctts" => "Composition time-to-sample",
            b"stss" => "Sync samples",
            b"mdat" => "Media data (video/audio)",
            b"free" => "Free space",
            b"skip" => "Skip",
            b"udta" => "User data (METADATA)",
            b"meta" => "Metadata (METADATA)",
            b"ilst" => "Item list (METADATA)",
            b"uuid" => "UUID extension (METADATA)",
            b"XMP_" => "XMP metadata (METADATA)",
            b"Xtra" => "Extra metadata (METADATA)",
            b"edts" => "Edit list container",
            b"elst" => "Edit list",
            _ => {
                let type_str = self.type_string();
                if type_str.starts_with('©') || type_str.starts_with('\u{a9}') {
                    "iTunes metadata (METADATA)"
                } else {
                    "Unknown"
                }
            }
        }
    }
}

/// Métadonnées extraites d'un fichier MP4
#[derive(Debug, Clone)]
pub struct Mp4Metadata {
    /// Titre (©nam)
    pub title: Option<String>,

    /// Artiste (©ART)
    pub artist: Option<String>,

    /// Album (©alb)
    pub album: Option<String>,

    /// Année (©day)
    pub year: Option<String>,

    /// Genre (©gen ou gnre)
    pub genre: Option<String>,

    /// Commentaire (©cmt)
    pub comment: Option<String>,

    /// Compositeur (©wrt)
    pub composer: Option<String>,

    /// Encodeur (©too)
    pub encoder: Option<String>,

    /// Copyright (cprt)
    pub copyright: Option<String>,

    /// Description (desc)
    pub description: Option<String>,

    /// Coordonnées GPS (©xyz) - DANGEREUX
    pub gps_coordinates: Option<String>,

    /// Image de couverture présente (covr)
    pub has_cover_art: bool,

    /// Atoms de métadonnées trouvés
    pub metadata_atoms: Vec<String>,

    /// Taille totale des métadonnées
    pub total_metadata_size: u64,
}

impl Mp4Metadata {
    pub fn new() -> Self {
        Self {
            title: None,
            artist: None,
            album: None,
            year: None,
            genre: None,
            comment: None,
            composer: None,
            encoder: None,
            copyright: None,
            description: None,
            gps_coordinates: None,
            has_cover_art: false,
            metadata_atoms: Vec::new(),
            total_metadata_size: 0,
        }
    }

    /// Vérifie si des métadonnées sont présentes
    pub fn has_metadata(&self) -> bool {
        self.title.is_some()
            || self.artist.is_some()
            || self.album.is_some()
            || self.year.is_some()
            || self.genre.is_some()
            || self.comment.is_some()
            || self.composer.is_some()
            || self.encoder.is_some()
            || self.copyright.is_some()
            || self.description.is_some()
            || self.gps_coordinates.is_some()
            || self.has_cover_art
            || !self.metadata_atoms.is_empty()
    }

    /// Compte le nombre de champs remplis
    pub fn field_count(&self) -> usize {
        let mut count = 0;
        if self.title.is_some() { count += 1; }
        if self.artist.is_some() { count += 1; }
        if self.album.is_some() { count += 1; }
        if self.year.is_some() { count += 1; }
        if self.genre.is_some() { count += 1; }
        if self.comment.is_some() { count += 1; }
        if self.composer.is_some() { count += 1; }
        if self.encoder.is_some() { count += 1; }
        if self.copyright.is_some() { count += 1; }
        if self.description.is_some() { count += 1; }
        if self.gps_coordinates.is_some() { count += 1; }
        if self.has_cover_art { count += 1; }
        count
    }
}

impl Default for Mp4Metadata {
    fn default() -> Self {
        Self::new()
    }
}
