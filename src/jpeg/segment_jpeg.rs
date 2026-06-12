/// Représente un segment JPEG
#[derive(Debug, Clone)]
pub struct JpegSegment {
    pub marker: u8,
    pub data: Vec<u8>,
}

impl JpegSegment {
    pub fn new(marker: u8, data: Vec<u8>) -> Self {
        Self { marker, data }
    }
    /// Retourne la taille totale du segment en bytes
    pub fn total_size(&self) -> usize {
        if is_standalone_marker(self.marker) {
            return 2;
        }

        2 + 2 + self.data.len()
    }
    pub fn marker_name(&self) -> &'static str {
    
        match self.marker {
            0xD8 => "SOI (Start Of Image)",
            0xD9 => "EOI (End Of Image)",
            0xE0 => "APP0 (JFIF)",
            0xE1 => "APP1 (EXIF)",
            0xE2 => "APP2 (ICC Profile)",
            0xED => "APP13 (Photoshop)",
            0xEE => "APP14 (Adobe)",
            0xFE => "COM (Comment)",
            0xDB => "DQT (Quantization Table)",
            0xC0 => "SOF0 (Start Of Frame - Baseline)",
            0xC1 => "SOF1 (Start Of Frame - Extended)",
            0xC2 => "SOF2 (Start Of Frame - Progressive)",
            0xC4 => "DHT (Huffman Table)",
            0xDA => "SOS (Start Of Scan)",
            0xDD => "DRI (Restart Interval)",
            _ if self.marker >= 0xD0 && self.marker <= 0xD7 => "RSTn (Restart Marker)",
            _ => "UNKNOWN",
        }
    }
}

fn is_standalone_marker(marker: u8) -> bool {
    marker == 0xD8      // SOI
    || marker == 0xD9   // EOI
    || marker == 0x01   // TEM
    || (0xD0..=0xD7).contains(&marker)  // RSTn
}

