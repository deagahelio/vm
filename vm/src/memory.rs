use std::sync::{Arc, Mutex};

pub struct Memory {
    pub bytes: Vec<u8>,
    pub framebuffer: Option<Arc<Mutex<Vec<u32>>>>,
    pub framebuffer_queue: Vec<(u32, u8)>
}

impl Memory {
    pub fn new(size: usize, framebuffer: Option<Arc<Mutex<Vec<u32>>>>) -> Self {
        Self {
            bytes: vec![0; size],
            framebuffer,
            framebuffer_queue: vec![],
        }
    }

    pub fn try_write_framebuffer(&mut self) {
        if let Ok(ref mut framebuffer) = self.framebuffer.as_ref().unwrap().try_lock() {
            for (address, value) in &self.framebuffer_queue {
                let offset = *address as usize - 0x100000;
                let mask = !(0xFF000000 >> (offset % 4 * 8));
                let value = (*value as u32) << (24 - (offset % 4 * 8));

                let pixel = &mut framebuffer[offset / 4];
                *pixel = (*pixel & mask) | value;
            }
            self.framebuffer_queue.clear();
        }
    }

    pub fn read_u8(&self, address: u32) -> Option<u8> {
        self.bytes.get(address as usize).copied()
    }

    pub fn read_u16(&self, address: u32) -> Option<u16> {
        Some(u16::from_le_bytes([self.read_u8(address)?,
                                 self.read_u8(address + 1)?]))
    }

    pub fn read_u32(&self, address: u32) -> Option<u32> {
        Some(u32::from_le_bytes([self.read_u8(address)?,
                                 self.read_u8(address + 1)?,
                                 self.read_u8(address + 2)?,
                                 self.read_u8(address + 3)?]))
    }

    pub fn write_u8(&mut self, address: u32, value: u8) -> Option<u8> {
        if self.framebuffer.is_some() && (0x100000..0x1fffffc).contains(&address) {
            self.framebuffer_queue.push((address, value));
        }
        self.bytes.get_mut(address as usize).map(|old_value| std::mem::replace(old_value, value))
    }

    pub fn write_u16(&mut self, address: u32, value: u16) -> Option<u16> {
        let value = u16::to_le_bytes(value);
        Some(u16::from_le_bytes([self.write_u8(address, value[0])?,
                                 self.write_u8(address + 1, value[1])?]))
    }

    pub fn write_u32(&mut self, address: u32, value: u32) -> Option<u32> {
        let value = u32::to_le_bytes(value);
        Some(u32::from_le_bytes([self.write_u8(address, value[0])?,
                                 self.write_u8(address + 1, value[1])?,
                                 self.write_u8(address + 2, value[2])?,
                                 self.write_u8(address + 3, value[3])?]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write() {
        let mut mem = Memory::new(16, None);

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