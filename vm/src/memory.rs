use std::convert::TryInto;

pub struct Memory {
    pub bytes: Vec<u8>,
}

impl Memory {
    pub fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
        }
    }

    pub fn read_u8(&self, address: u32) -> Option<u8> {
        self.bytes.get(address as usize).copied()
    }

    pub fn read_u16(&self, address: u32) -> Option<u16> {
        self.bytes.get(address as usize..address as usize + 2).map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn read_u32(&self, address: u32) -> Option<u32> {
        self.bytes.get(address as usize..address as usize + 4).map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub fn write_u8(&mut self, address: u32, value: u8) -> Option<u8> {
        self.bytes.get_mut(address as usize).map(|old_value| std::mem::replace(old_value, value))
    }

    pub fn write_u16(&mut self, address: u32, value: u16) -> Option<u16> {
        self.bytes.get_mut(address as usize..address as usize + 2).map(|old_bytes| 
            u16::from_le_bytes(std::mem::replace(old_bytes.try_into().unwrap(), u16::to_le_bytes(value)))
        )
    }

    pub fn write_u32(&mut self, address: u32, value: u32) -> Option<u32> {
        self.bytes.get_mut(address as usize..address as usize + 4).map(|old_bytes| 
            u32::from_le_bytes(std::mem::replace(old_bytes.try_into().unwrap(), u32::to_le_bytes(value)))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write() {
        let mut mem = Memory::new(16);

        assert_eq!(mem.write_u8(2, 0x12), Some(0));
        assert_eq!(mem.write_u8(1, 0x34), Some(0));
        assert_eq!(mem.read_u8(2), Some(0x12));
        assert_eq!(mem.read_u8(1), Some(0x34));
        assert_eq!(mem.read_u32(0), Some(0x123400));
        assert_eq!(mem.write_u32(3, 0xAABBCCDD), Some(0));
        assert_eq!(mem.read_u32(2), Some(0xBBCCDD12));
        assert_eq!(mem.write_u32(1, 0), Some(0xCCDD1234));

        assert_eq!(mem.read_u32(12), Some(0));
        assert_eq!(mem.read_u32(13), None);
        assert_eq!(mem.read_u8(16), None);
        assert_eq!(mem.read_u8(15), Some(0));
    }
}