/// Représente un chunk RIFF/WAV
#[derive(Debug, Clone)]
pub struct WavChunk {
    pub chunk_id: [u8; 4],
    pub offset: usize,
    pub data_size: u32,
}

impl WavChunk {
    pub fn new(chunk_id: [u8; 4], offset: usize, data_size: u32) -> Self {
        Self {
            chunk_id,
            offset,
            data_size,
        }
    }
    pub fn total_size(&self) -> usize {
        8 + self.data_size as usize
    }
    pub fn id_string(&self) -> String {
        self.chunk_id
            .iter()
            .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '?' })
            .collect()
    }
    pub fn is_metadata(&self) -> bool {
        matches!(
            &self.chunk_id,
            b"LIST" | b"id3 " | b"ID3 " | b"bext" | b"BEXT"
            | b"cart" | b"CART" | b"cue " | b"CUE "
            | b"plst" | b"PLST" | b"smpl" | b"SMPL"
            | b"inst" | b"INST" | b"ltxt" | b"LTXT"
            | b"note" | b"NOTE" | b"labl" | b"LABL"
            | b"_PMX" | b"iXML" | b"IXML" | b"axml" | b"AXML"
            | b"afsp" | b"AFSP"
        )
    }

    pub fn is_essential(&self) -> bool {
        matches!(
            &self.chunk_id,
            b"fmt " | b"FMT " | b"data" | b"DATA" | b"fact" | b"FACT"
        )
    }

    pub fn description(&self) -> &'static str {
        match &self.chunk_id {
            b"fmt " | b"FMT " => "Format (audio parameters)",
            b"data" | b"DATA" => "Audio data",
            b"fact" | b"FACT" => "Fact (sample count)",
            b"LIST" => "LIST (metadata container)",
            b"id3 " | b"ID3 " => "ID3 tag (metadata)",
            b"bext" | b"BEXT" => "Broadcast Extension (metadata)",
            b"cart" | b"CART" => "Cart chunk (radio metadata)",
            b"cue " | b"CUE " => "Cue points",
            b"plst" | b"PLST" => "Playlist",
            b"smpl" | b"SMPL" => "Sample info",
            b"inst" | b"INST" => "Instrument info",
            b"ltxt" | b"LTXT" => "Label text",
            b"note" | b"NOTE" => "Note",
            b"labl" | b"LABL" => "Label",
            b"_PMX" => "XMP metadata",
            b"iXML" | b"IXML" => "iXML metadata",
            b"axml" | b"AXML" => "AXML metadata",
            b"afsp" | b"AFSP" => "AFsp metadata",
            b"JUNK" | b"junk" => "Padding (junk)",
            b"PAD " | b"pad " => "Padding",
            _ => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WavMetadata {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub album: Option<String>,
    pub creation_date: Option<String>,
    pub genre: Option<String>,
    pub comment: Option<String>,
    pub software: Option<String>,
    pub copyright: Option<String>,
    pub engineer: Option<String>,
    pub technician: Option<String>,
    pub source: Option<String>,
    pub has_bext: bool,
    pub has_id3: bool,
    pub has_xmp: bool,
    pub metadata_chunks: Vec<String>,
    pub total_metadata_size: u32,
}

impl WavMetadata {
    pub fn new() -> Self {
        Self {
            artist: None,
            title: None,
            album: None,
            creation_date: None,
            genre: None,
            comment: None,
            software: None,
            copyright: None,
            engineer: None,
            technician: None,
            source: None,
            has_bext: false,
            has_id3: false,
            has_xmp: false,
            metadata_chunks: Vec::new(),
            total_metadata_size: 0,
        }
    }

    /// Vérifie si des métadonnées sont présentes
    pub fn has_metadata(&self) -> bool {
        self.artist.is_some()
            || self.title.is_some()
            || self.album.is_some()
            || self.creation_date.is_some()
            || self.genre.is_some()
            || self.comment.is_some()
            || self.software.is_some()
            || self.copyright.is_some()
            || self.engineer.is_some()
            || self.technician.is_some()
            || self.source.is_some()
            || self.has_bext
            || self.has_id3
            || self.has_xmp
            || !self.metadata_chunks.is_empty()
    }

    /// Compte le nombre de champs remplis
    pub fn field_count(&self) -> usize {
        let mut count = 0;
        if self.artist.is_some() { count += 1; }
        if self.title.is_some() { count += 1; }
        if self.album.is_some() { count += 1; }
        if self.creation_date.is_some() { count += 1; }
        if self.genre.is_some() { count += 1; }
        if self.comment.is_some() { count += 1; }
        if self.software.is_some() { count += 1; }
        if self.copyright.is_some() { count += 1; }
        if self.engineer.is_some() { count += 1; }
        if self.technician.is_some() { count += 1; }
        if self.source.is_some() { count += 1; }
        if self.has_bext { count += 1; }
        if self.has_id3 { count += 1; }
        if self.has_xmp { count += 1; }
        count
    }
}

impl Default for WavMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Informations sur le format audio WAV
#[derive(Debug, Clone)]
pub struct WavFormat {
    pub audio_format: u16,

    pub num_channels: u16,

    pub sample_rate: u32,

    pub byte_rate: u32,

    pub block_align: u16,

    pub bits_per_sample: u16,
}

impl WavFormat {
    pub fn format_name(&self) -> &'static str {
        match self.audio_format {
            1 => "PCM (uncompressed)",
            2 => "Microsoft ADPCM",
            3 => "IEEE Float",
            6 => "A-law",
            7 => "mu-law",
            0xFFFE => "Extensible",
            _ => "Unknown",
        }
    }

    pub fn channels_name(&self) -> &'static str {
        match self.num_channels {
            1 => "Mono",
            2 => "Stereo",
            6 => "5.1 Surround",
            8 => "7.1 Surround",
            _ => "Multi-channel",
        }
    }
}
