use crate::device::{WriteResult, DeviceRecord, Class, Device};
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Bytes {
    pub bytes: Vec<u8>,
}

impl Bytes {
    pub fn new(size: usize) -> Self {
        Self {
            bytes: vec![0; size],
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

pub struct Memory {
    pub bytes: Bytes,
    pub devices: Vec<Rc<RefCell<dyn Device>>>,
}

impl Memory {
    pub fn new(size: usize, mut devices: Vec<Rc<RefCell<dyn Device>>>) -> Self {
        let mut bytes = Bytes::new(size);

        for mut device in devices.iter_mut().filter_map(|dev| dev.try_borrow_mut().ok()) {
            device.init_memory(&mut bytes);
        }

        Self {
            bytes,
            devices,
        }
    }

    pub fn load_boot(&mut self, file_name: &str) -> std::io::Result<()> {
        let mut file = File::open(file_name)?;
        file.read(&mut self.bytes.bytes[0x200..])?;
        Ok(())
    }

    pub fn read_u8(&mut self, address: u32) -> Option<u8> {
        for mut device in self.devices.iter_mut().filter_map(|dev| dev.try_borrow_mut().ok()) {
            if device.get_memory_area().contains(&address) {
                return device.read_memory(&mut self.bytes, address);
            }
        }

        self.bytes.read_u8(address)
    }

    pub fn read_u16(&mut self, address: u32) -> Option<u16> {
        Some(u16::from_le_bytes([self.read_u8(address)?,
                                 self.read_u8(address + 1)?]))
    }

    pub fn read_u32(&mut self, address: u32) -> Option<u32> {
        Some(u32::from_le_bytes([self.read_u8(address)?,
                                 self.read_u8(address + 1)?,
                                 self.read_u8(address + 2)?,
                                 self.read_u8(address + 3)?]))
    }

    pub fn write_u8(&mut self, address: u32, value: u8) -> Option<u8> {
        for mut device in self.devices.iter().filter_map(|dev| dev.try_borrow_mut().ok()) {
            if device.get_memory_area().contains(&address) {
                return match device.write_memory(&mut self.bytes, address, value) {
                    WriteResult::Write => self.bytes.write_u8(address, value),
                    WriteResult::Cancel => self.bytes.read_u8(address),
                };
            }
        }

        self.bytes.write_u8(address, value)
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

pub struct MemoryDevice {
    record: DeviceRecord,
}

impl MemoryDevice {
    pub fn new(id: u8, memory_size: usize) -> Self {
        Self {
            record: DeviceRecord {
                id,
                class: Class::Memory,
                limit_0: memory_size as u32,
                .. Default::default()
            },
        }
    }
}

impl Device for MemoryDevice {
    fn get_record(&self) -> Option<&DeviceRecord> {
        Some(&self.record)
    }

    fn get_memory_area(&self) -> std::ops::RangeInclusive<u32> {
        1..=0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write() {
        let mut bytes = Bytes::new(16);

        assert_eq!(bytes.write_u8(2, 0x12), Some(0));
        assert_eq!(bytes.write_u8(1, 0x34), Some(0));
        assert_eq!(bytes.read_u8(2), Some(0x12));
        assert_eq!(bytes.read_u8(1), Some(0x34));
        assert_eq!(bytes.read_u32(0), Some(0x123400));
        assert_eq!(bytes.write_u32(3, 0xAABBCCDD), Some(0));
        assert_eq!(bytes.read_u32(2), Some(0xBBCCDD12));
        assert_eq!(bytes.write_u32(1, 0), Some(0xCCDD1234));

        assert_eq!(bytes.read_u32(12), Some(0));
        assert_eq!(bytes.read_u32(13), None);
        assert_eq!(bytes.read_u8(16), None);
        assert_eq!(bytes.read_u8(15), Some(0));
    }
}