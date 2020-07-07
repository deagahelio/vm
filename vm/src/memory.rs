use crate::device::{Device, Class};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct Memory {
    pub bytes: Vec<u8>,
    pub framebuffer: Option<Arc<Mutex<Vec<u32>>>>,
    pub framebuffer_queue: Vec<(u32, u8)>,
    pub devices: HashMap<u8, Device>,
    device_id: u8,
    last_device_id: u8,
    pub disks: [Option<Vec<u8>>; 8],
    disk_controller_input: [u8; 4],
    selected_disk: usize,
}

impl Memory {
    pub fn new(size: usize, framebuffer: Option<Arc<Mutex<Vec<u32>>>>) -> Self {
        let mut m = Self {
            bytes: vec![0; size],
            framebuffer,
            framebuffer_queue: vec![],
            devices: HashMap::new(),
            device_id: 0,
            last_device_id: 0,
            disks: Default::default(),
            disk_controller_input: [0; 4],
            selected_disk: 0,
        };
        m.register_device(Device { class: Class::Memory, limit_0: size as u32, .. Default::default() });
        m.register_device(Device { class: Class::Monitor, base_address_0: 0x2100000, base_address_1: 0x100000, limit_1: 0x20FFFFF, .. Default::default() });
        m.register_device(Device { class: Class::DiskController, base_address_0: 0xF1200, base_address_1: 0xF1000, limit_1: 0xF11FF, .. Default::default() });
        m.unchecked_write_u8(0xF0000, 0x01);
        m.unchecked_write_u8(0xF1200, 0x01);
        m
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

    pub fn register_device(&mut self, mut device: Device) -> u8 {
        self.last_device_id += 1;
        device.id = self.last_device_id;
        self.devices.insert(self.last_device_id, device);
        self.last_device_id
    }

    pub fn set_disk(&mut self, slot: usize, disk: Option<Vec<u8>>) {
        self.disks[slot] = disk;
        let mut register = 0;
        for (i, slot) in self.disks.iter().enumerate() {
            if slot.is_some() {
                register |= 1 << i;
            }
        }
        self.unchecked_write_u8(0xF1201, register);
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

    fn unchecked_write_u8(&mut self, address: u32, value: u8) -> Option<u8> {
        self.bytes.get_mut(address as usize).map(|old_value| std::mem::replace(old_value, value))
    }

    fn unchecked_write_u16(&mut self, address: u32, value: u16) -> Option<u16> {
        let value = u16::to_le_bytes(value);
        Some(u16::from_le_bytes([self.unchecked_write_u8(address, value[0])?,
                                 self.unchecked_write_u8(address + 1, value[1])?]))
    }

    fn unchecked_write_u32(&mut self, address: u32, value: u32) -> Option<u32> {
        let value = u32::to_le_bytes(value);
        Some(u32::from_le_bytes([self.unchecked_write_u8(address, value[0])?,
                                 self.unchecked_write_u8(address + 1, value[1])?,
                                 self.unchecked_write_u8(address + 2, value[2])?,
                                 self.unchecked_write_u8(address + 3, value[3])?]))
    }

    pub fn write_u8(&mut self, address: u32, value: u8) -> Option<u8> {
        if self.framebuffer.is_some() && (0x100000..0x2100000).contains(&address) {
            self.framebuffer_queue.push((address, value));
        }

        match address {
            0xF0001 => self.device_id = value,
            0xF0000 if value == 0x01 => {
                if let Some(device) = self.devices.get(&self.device_id) {
                    let device = device.clone();
                    self.unchecked_write_u8(0xF0000, 0x01);
                    self.unchecked_write_u8(0xF0001, self.device_id);
                    self.unchecked_write_u8(0xF0002, device.class as u8);
                    self.unchecked_write_u8(0xF0003, device.interrupt_line);
                    self.unchecked_write_u32(0xF0004, device.base_address_0);
                    self.unchecked_write_u32(0xF0008, device.limit_0);
                    self.unchecked_write_u32(0xF000C, device.base_address_1);
                    self.unchecked_write_u32(0xF0010, device.limit_1);
                } else {
                    self.unchecked_write_u8(0xF0000, 0x04);
                }
            },
            0xF1200 if value == 0x1 => {
                let address = u32::from_le_bytes(self.disk_controller_input) as usize * 512;
                match self.disks[self.selected_disk].as_ref().unwrap().get(address..address + 512) {
                    Some(sector) => self.bytes[0xF1000..0xF1200].copy_from_slice(sector),
                    None => {
                        self.unchecked_write_u8(0xF1200, 0x04);
                        self.unchecked_write_u8(0xF1202, 0x02);
                    },
                }
            },
            0xF1200 if value == 0x2 => {
                let address = u32::from_le_bytes(self.disk_controller_input) as usize * 512;
                match self.disks[self.selected_disk].as_mut().unwrap().get_mut(address..address + 512) {
                    Some(sector) => sector.copy_from_slice(&self.bytes[0xF1000..0xF1200]),
                    None => {
                        self.unchecked_write_u8(0xF1200, 0x04);
                        self.unchecked_write_u8(0xF1202, 0x02);
                    },
                }
            },
            0xF1200 if value == 0x4 => {
                match self.disks.get(self.disk_controller_input[0] as usize) {
                    Some(Some(_)) => {
                        self.unchecked_write_u8(0xF1200, 0x01);
                        self.selected_disk = self.disk_controller_input[0] as usize;
                    },
                    _ => {
                        self.unchecked_write_u8(0xF1200, 0x04);
                        self.unchecked_write_u8(0xF1202, 0x01);
                    },
                }
            },
            0xF1200 if value == 0x8 => {
                self.unchecked_write_u32(0xF1203, self.disks[self.selected_disk].as_ref().unwrap().len() as u32 / 512);
            },
            0xF1201..=0xF1204 => self.disk_controller_input[address as usize - 0xF1201] = value,
            _ => {},
        }

        if (0xF0000..=0xF0013).contains(&address) || (0xF1200..=0xF1206).contains(&address) {
            self.read_u8(address)
        } else {
            self.unchecked_write_u8(address, value)
        }
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