use crate::device::{Device, DeviceRecord, Class, WriteResult};
use crate::memory::Bytes;

pub struct InterruptController {
    record: DeviceRecord,
    address: u32,
    table_address: u32,
    interrupts_enabled: bool,
    interrupts_bitmask: u16,
}

impl InterruptController {
    pub fn new(id: u8, address: u32) -> Self {
        Self {
            record: DeviceRecord {
                id,
                class: Class::InterruptController,
                base_address_0: address + 64,
                base_address_1: address,
                limit_1: address + 64 - 1,
                .. Default::default()
            },
            address: address + 64,
            table_address: address,
            interrupts_enabled: false,
            interrupts_bitmask: 0b11111111_11111111,
        }
    }
}

impl Device for InterruptController {
    fn get_record(&self) -> Option<&DeviceRecord> {
        Some(&self.record)
    }

    fn get_memory_area(&self) -> std::ops::RangeInclusive<u32> {
        self.table_address..=self.address + 2
    }

    fn init_memory(&mut self, bytes: &mut Bytes) {
        bytes.write_u8(self.address, 0x01);
        bytes.write_u16(self.address + 1, self.interrupts_bitmask);
    }

    fn write_memory(&mut self, bytes: &mut Bytes, address: u32, value: u8) -> WriteResult {
        match address - self.address {
            0 => {
                self.interrupts_enabled = value & 1 == 1;
            },
            1..=2 => {
                self.interrupts_bitmask = bytes.read_u16(address + 1).unwrap();
            },
            _ => {},
        }

        WriteResult::Write
    }
}