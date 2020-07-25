use crate::device::{Class, DeviceRecord, Device};
use crate::memory::Bytes;

pub struct Monitor {
    record: DeviceRecord,
    address: u32,
    framebuffer_address: u32,
    framebuffer_size: u32,
}

impl Monitor {
    pub fn new(id: u8, address: u32, framebuffer_size: u32) -> Self {
        Self {
            record: DeviceRecord {
                id,
                class: Class::Monitor,
                base_address_0: address + framebuffer_size,
                base_address_1: address,
                limit_1: address + framebuffer_size - 1,
                .. Default::default()
            },
            address: address + framebuffer_size,
            framebuffer_address: address,
            framebuffer_size,
        }
    }
}

impl Device for Monitor {
    fn get_record(&self) -> Option<&DeviceRecord> {
        Some(&self.record)
    }

    fn get_memory_area(&self) -> std::ops::RangeInclusive<u32> {
        self.framebuffer_address..=self.address
    }

    fn init_memory(&mut self, bytes: &mut Bytes) {
        bytes.write_u8(self.address, 0x01);
    }
}