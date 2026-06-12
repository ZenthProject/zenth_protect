#[derive(Debug)]
pub struct Chunk {
    pub length: u32,
    pub chunk_type: [u8; 4], 
}

