pub struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0}
    }


    pub fn read_u8(&mut self) -> Option<u8> {
        if self.pos >= self.data.len() {
            return None;
        }
        let value = self.data[self.pos];
        self.pos +=1;
        Some(value)
    }

    pub fn read_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.pos + n > self.data.len() {
            return None; 
        }
        let slice = &self.data[self.pos..self.pos +n];
        self.pos += n;
        Some(slice)
    }

    pub fn read_u16_be(&mut self) -> Option<u16> {
        let bytes = self.read_bytes(2)?;
        let arr = [bytes[0] , bytes[1]];
        let value = u16::from_be_bytes(arr);
        Some(value)
    }

    pub fn read_u32_be(&mut self) -> Option<u32> {
        let bytes = self.read_bytes(4)?;
        let arr = [bytes[0],bytes[1],bytes[2],bytes[3]];
        let value = u32::from_be_bytes(arr);
        Some(value)
    }

    pub fn read_u32_le(&mut self) -> Option<u32> {
        let bytes = self.read_bytes(4)?;
        let arr = [bytes[0], bytes[1], bytes[2], bytes[3]];
        let value = u32::from_le_bytes(arr);
        Some(value)
    }

     pub fn skip(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.data.len());
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x42, 0xFF];
        let mut cur = Cursor::new(&data);
        assert_eq!(cur.read_u8(), Some(0x42));
        assert_eq!(cur.read_u8(), Some(0xFF));
        assert_eq!(cur.read_u8(), None);
    }

    #[test]
    fn test_read_bytes() {
        let data = [1, 2, 3, 4, 5];
        let mut cur = Cursor::new(&data);
        assert_eq!(cur.read_bytes(3), Some(&[1, 2, 3][..]));
        assert_eq!(cur.read_bytes(3), None);
        assert_eq!(cur.read_bytes(2), Some(&[4, 5][..]));
    }

    #[test]
    fn test_read_u16_be() {
        let data = [0x12, 0x34];
        let mut cur = Cursor::new(&data);
        assert_eq!(cur.read_u16_be(), Some(0x1234));
    }

    #[test]
    fn test_read_u32_be() {
        let data = [0x12, 0x34, 0x56, 0x78];
        let mut cur = Cursor::new(&data);
        assert_eq!(cur.read_u32_be(), Some(0x12345678));
    }

    #[test]
    fn test_read_u32_le() {
        let data = [0x78, 0x56, 0x34, 0x12];
        let mut cur = Cursor::new(&data);
        assert_eq!(cur.read_u32_le(), Some(0x12345678));
    }

    #[test]
    fn test_skip() {
        let data = [1, 2, 3, 4, 5];
        let mut cur = Cursor::new(&data);
        cur.skip(2);
        assert_eq!(cur.read_u8(), Some(3));
        cur.skip(100);
        assert_eq!(cur.remaining(), 0);
    }

    #[test]
    fn test_remaining() {
        let data = [1, 2, 3];
        let mut cur = Cursor::new(&data);
        assert_eq!(cur.remaining(), 3);
        cur.read_u8();
        assert_eq!(cur.remaining(), 2);
    }

    #[test]
    fn test_empty_buffer() {
        let data: [u8; 0] = [];
        let mut cur = Cursor::new(&data);
        assert_eq!(cur.read_u8(), None);
        assert_eq!(cur.read_u16_be(), None);
        assert_eq!(cur.read_u32_be(), None);
    }
}