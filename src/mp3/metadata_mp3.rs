/// Représente les métadonnées d'un fichier MP3
///
/// Combine les informations de ID3v1 et ID3v2

#[derive(Debug, Clone, PartialEq)]
pub struct Id3v2Metadata {
    pub version_major: u8,

    pub version_minor: u8,

    pub tag_size: u32,

    pub title: Option<String>,

    pub artist: Option<String>,

    pub album: Option<String>,

    pub year: Option<String>,

    pub track: Option<String>,

    pub genre: Option<String>,

    pub comment: Option<String>,

    pub composer: Option<String>,
    pub encoder: Option<String>,
    pub has_picture: bool,

    pub custom_fields: Vec<(String, String)>,

    pub has_private_data: bool,

    pub has_embedded_objects: bool,
}

impl Id3v2Metadata {
    pub fn new() -> Self {
        Self {
            version_major: 0,
            version_minor: 0,
            tag_size: 0,
            title: None,
            artist: None,
            album: None,
            year: None,
            track: None,
            genre: None,
            comment: None,
            composer: None,
            encoder: None,
            has_picture: false,
            custom_fields: Vec::new(),
            has_private_data: false,
            has_embedded_objects: false,
        }
    }

    pub fn has_metadata(&self) -> bool {
        self.title.is_some()
            || self.artist.is_some()
            || self.album.is_some()
            || self.year.is_some()
            || self.track.is_some()
            || self.genre.is_some()
            || self.comment.is_some()
            || self.composer.is_some()
            || self.encoder.is_some()
            || self.has_picture
            || !self.custom_fields.is_empty()
            || self.has_private_data
            || self.has_embedded_objects
    }

    pub fn field_count(&self) -> usize {
        let mut count = 0;
        if self.title.is_some() { count += 1; }
        if self.artist.is_some() { count += 1; }
        if self.album.is_some() { count += 1; }
        if self.year.is_some() { count += 1; }
        if self.track.is_some() { count += 1; }
        if self.genre.is_some() { count += 1; }
        if self.comment.is_some() { count += 1; }
        if self.composer.is_some() { count += 1; }
        if self.encoder.is_some() { count += 1; }
        if self.has_picture { count += 1; }
        if self.has_private_data { count += 1; }
        if self.has_embedded_objects { count += 1; }
        count + self.custom_fields.len()
    }
}

impl Default for Id3v2Metadata {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Id3v1Metadata {
    pub title: Option<String>,

    pub artist: Option<String>,

    pub album: Option<String>,

    pub year: Option<String>,

    pub comment: Option<String>,

    pub track: Option<u8>,

    pub genre_id: Option<u8>,
}

impl Id3v1Metadata {
    pub fn new() -> Self {
        Self {
            title: None,
            artist: None,
            album: None,
            year: None,
            comment: None,
            track: None,
            genre_id: None,
        }
    }

    pub fn has_metadata(&self) -> bool {
        self.title.is_some()
            || self.artist.is_some()
            || self.album.is_some()
            || self.year.is_some()
            || self.comment.is_some()
            || self.track.is_some()
    }

    pub fn genre_name(&self) -> Option<&'static str> {
        self.genre_id.and_then(|id| ID3V1_GENRES.get(id as usize).copied())
    }
}

impl Default for Id3v1Metadata {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Mp3Metadata {
    pub id3v2: Option<Id3v2Metadata>,

    pub id3v1: Option<Id3v1Metadata>,

    pub has_ape_tag: bool,
}

impl Mp3Metadata {
    pub fn new() -> Self {
        Self {
            id3v2: None,
            id3v1: None,
            has_ape_tag: false,
        }
    }

    pub fn has_metadata(&self) -> bool {
        self.id3v2.as_ref().is_some_and(|m| m.has_metadata())
            || self.id3v1.as_ref().is_some_and(|m| m.has_metadata())
            || self.has_ape_tag
    }
}

impl Default for Mp3Metadata {
    fn default() -> Self {
        Self::new()
    }
}

pub static ID3V1_GENRES: &[&str] = &[
    "Blues", "Classic Rock", "Country", "Dance", "Disco", "Funk", "Grunge",
    "Hip-Hop", "Jazz", "Metal", "New Age", "Oldies", "Other", "Pop", "R&B",
    "Rap", "Reggae", "Rock", "Techno", "Industrial", "Alternative", "Ska",
    "Death Metal", "Pranks", "Soundtrack", "Euro-Techno", "Ambient",
    "Trip-Hop", "Vocal", "Jazz+Funk", "Fusion", "Trance", "Classical",
    "Instrumental", "Acid", "House", "Game", "Sound Clip", "Gospel",
    "Noise", "AlternRock", "Bass", "Soul", "Punk", "Space", "Meditative",
    "Instrumental Pop", "Instrumental Rock", "Ethnic", "Gothic", "Darkwave",
    "Techno-Industrial", "Electronic", "Pop-Folk", "Eurodance", "Dream",
    "Southern Rock", "Comedy", "Cult", "Gangsta", "Top 40", "Christian Rap",
    "Pop/Funk", "Jungle", "Native American", "Cabaret", "New Wave",
    "Psychedelic", "Rave", "Showtunes", "Trailer", "Lo-Fi", "Tribal",
    "Acid Punk", "Acid Jazz", "Polka", "Retro", "Musical", "Rock & Roll",
    "Hard Rock", "Folk", "Folk-Rock", "National Folk", "Swing", "Fast Fusion",
    "Bebop", "Latin", "Revival", "Celtic", "Bluegrass", "Avantgarde",
    "Gothic Rock", "Progressive Rock", "Psychedelic Rock", "Symphonic Rock",
    "Slow Rock", "Big Band", "Chorus", "Easy Listening", "Acoustic",
    "Humour", "Speech", "Chanson", "Opera", "Chamber Music", "Sonata",
    "Symphony", "Booty Bass", "Primus", "Porn Groove", "Satire", "Slow Jam",
    "Club", "Tango", "Samba", "Folklore", "Ballad", "Power Ballad",
    "Rhythmic Soul", "Freestyle", "Duet", "Punk Rock", "Drum Solo",
    "A capella", "Euro-House", "Dance Hall", "Goa", "Drum & Bass",
    "Club-House", "Hardcore Techno", "Terror", "Indie", "BritPop",
    "Negerpunk", "Polsk Punk", "Beat", "Christian Gangsta Rap",
    "Heavy Metal", "Black Metal", "Crossover", "Contemporary Christian",
    "Christian Rock", "Merengue", "Salsa", "Thrash Metal", "Anime", "Jpop",
    "Synthpop", "Abstract", "Art Rock", "Baroque", "Bhangra", "Big Beat",
    "Breakbeat", "Chillout", "Downtempo", "Dub", "EBM", "Eclectic",
    "Electro", "Electroclash", "Emo", "Experimental", "Garage", "Global",
    "IDM", "Illbient", "Industro-Goth", "Jam Band", "Krautrock", "Leftfield",
    "Lounge", "Math Rock", "New Romantic", "Nu-Breakz", "Post-Punk",
    "Post-Rock", "Psytrance", "Shoegaze", "Space Rock", "Trop Rock",
    "World Music", "Neoclassical", "Audiobook", "Audio Theatre",
    "Neue Deutsche Welle", "Podcast", "Indie Rock", "G-Funk", "Dubstep",
    "Garage Rock", "Psybient",
];
