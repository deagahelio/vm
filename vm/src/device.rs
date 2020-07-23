use crate::memory::Bytes;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum Class {
    Unspecified = 0x0,
    Memory = 0x1,
    DiskController = 0x2,
    InterruptController = 0x3,
    Timer = 0x4,
    PowerManager = 0x5,
    Mouse = 0x10,
    Keyboard = 0x11,
    Monitor = 0x20,
}

impl Default for Class {
    fn default() -> Self {
        Self::Unspecified
    }
}

#[derive(Default, Clone)]
pub struct DeviceRecord {
    pub id: u8,
    pub class: Class,
    pub interrupt_line: u8,
    pub base_address_0: u32,
    pub limit_0: u32,
    pub base_address_1: u32,
    pub limit_1: u32,
}

pub enum WriteResult {
    Write,
    Cancel,
}

#[allow(unused_variables)]
pub trait Device {
    fn get_record(&self) -> Option<&DeviceRecord> { None }
    fn get_memory_area(&self) -> std::ops::RangeInclusive<u32>;
    fn init_memory(&mut self, bytes: &mut Bytes) {}
    fn read_memory(&mut self, bytes: &mut Bytes, address: u32) -> Option<u8> { bytes.read_u8(address) }
    fn write_memory(&mut self, bytes: &mut Bytes, address: u32, value: u8) -> WriteResult { WriteResult::Write }
}

pub struct DeviceManager {
    device_id: u8,
    pub records: HashMap<u8, DeviceRecord>,
    address: u32,
}

impl DeviceManager {
    pub fn new(address: u32) -> Self {
        Self {
            device_id: 0,
            records: HashMap::new(),
            address,
        }
    }

    pub fn register_record(&mut self, record: &DeviceRecord) {
        let record = record.clone();
        self.records.insert(record.id, record);
    }
}

impl Device for DeviceManager {
    fn get_memory_area(&self) -> std::ops::RangeInclusive<u32> {
        self.address..=self.address + 0x13
    }

    fn init_memory(&mut self, bytes: &mut Bytes) {
        bytes.write_u8(self.address, 0x01);
    }

    fn write_memory(&mut self, bytes: &mut Bytes, address: u32, value: u8) -> WriteResult {
        match address - self.address {
            0 if value == 0x01 => {
                if let Some(record) = self.records.get(&self.device_id) {
                    bytes.write_u8(self.address, 0x01);
                    bytes.write_u8(self.address + 0x1, self.device_id);
                    bytes.write_u8(self.address + 0x2, record.class as u8);
                    bytes.write_u8(self.address + 0x3, record.interrupt_line);
                    bytes.write_u32(self.address + 0x4, record.base_address_0);
                    bytes.write_u32(self.address + 0x8, record.limit_0);
                    bytes.write_u32(self.address + 0xC, record.base_address_1);
                    bytes.write_u32(self.address + 0x10, record.limit_1);
                } else {
                    bytes.write_u8(self.address, 0x04);
                }
            },
            1 => {
                self.device_id = value;
            },
            _ => {},
        }

        WriteResult::Cancel
    }
}