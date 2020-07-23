use crate::device::{Class, DeviceRecord, Device, WriteResult};
use crate::memory::Bytes;

pub struct DiskController {
    record: DeviceRecord,
    address: u32,
    data_address: u32,
    disks: [Option<Vec<u8>>; 8],
    input: [u8; 4],
    selected_disk: usize,
    update_disk_register: Option<u8>,
}

impl DiskController {
    pub fn new(id: u8, address: u32) -> Self {
        Self {
            record: DeviceRecord {
                id,
                class: Class::DiskController,
                base_address_0: address + 512,
                base_address_1: address,
                limit_1: address + 512 - 1,
                .. Default::default()
            },
            address: address + 512,
            data_address: address,
            disks: Default::default(),
            input: [0; 4],
            selected_disk: 0,
            update_disk_register: None,
        }
    }

    pub fn set_disk(&mut self, slot: usize, disk: Option<Vec<u8>>) {
        self.disks[slot] = disk;
        let mut register = 0;
        for (i, slot) in self.disks.iter().enumerate() {
            if slot.is_some() {
                register |= 1 << i;
            }
        }
        self.update_disk_register = Some(register);
    }
}

impl Device for DiskController {
    fn get_record(&self) -> Option<&DeviceRecord> {
        Some(&self.record)
    }

    fn get_memory_area(&self) -> std::ops::RangeInclusive<u32> {
        self.data_address..=self.address + 0x6
    }

    fn init_memory(&mut self, bytes: &mut Bytes) {
        bytes.write_u8(self.address, 0x01);
    }

    fn read_memory(&mut self, bytes: &mut Bytes, address: u32) -> Option<u8> {
        if let Some(register) = self.update_disk_register {
            bytes.write_u8(self.address + 1, register);
            self.update_disk_register = None;
        }
        bytes.read_u8(address)
    }

    fn write_memory(&mut self, bytes: &mut Bytes, address: u32, value: u8) -> WriteResult {
        match address - self.address {
            0 if value == 0x1 => {
                let address = u32::from_le_bytes(self.input) as usize * 512;
                match self.disks[self.selected_disk].as_ref().unwrap().get(address..address + 512) {
                    Some(sector) => bytes.bytes[self.data_address as usize..self.address as usize].copy_from_slice(sector),
                    None => {
                        bytes.write_u8(self.address, 0x04);
                        bytes.write_u8(self.address + 2, 0x02);
                    },
                }
            },
            0 if value == 0x2 => {
                let address = u32::from_le_bytes(self.input) as usize * 512;
                match self.disks[self.selected_disk].as_mut().unwrap().get_mut(address..address + 512) {
                    Some(sector) => sector.copy_from_slice(&bytes.bytes[self.data_address as usize..self.address as usize]),
                    None => {
                        bytes.write_u8(self.address, 0x04);
                        bytes.write_u8(self.address + 2, 0x02);
                    },
                }
            },
            0 if value == 0x4 => {
                match self.disks.get(self.input[0] as usize) {
                    Some(Some(_)) => {
                        bytes.write_u8(self.address, 0x01);
                        self.selected_disk = self.input[0] as usize;
                    },
                    _ => {
                        bytes.write_u8(self.address, 0x04);
                        bytes.write_u8(self.address + 2, 0x01);
                    },
                }
            },
            0 if value == 0x8 => {
                bytes.write_u32(self.address + 3, self.disks[self.selected_disk].as_ref().unwrap().len() as u32 / 512);
            },
            1..=4 => {
                self.input[address as usize - (self.address + 1) as usize] = value;
            },
            _ => {},
        }

        WriteResult::Cancel
    }
}