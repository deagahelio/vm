use crate::device::{DeviceRecord, Class, Device, WriteResult};
use crate::memory::Bytes;
use crate::interrupt_controller::InterruptQueue;
use minifb::Key;
use std::sync::mpsc::Receiver;

pub struct Keyboard {
    record: DeviceRecord,
    address: u32,
    interrupt_queue: InterruptQueue,
    key_receiver: Receiver<Key>,
    waiting: bool,
}

impl Keyboard {
    pub fn new(id: u8, address: u32, interrupt_queue: InterruptQueue, key_receiver: Receiver<Key>) -> Self {
        Self {
            record: DeviceRecord {
                id,
                class: Class::Keyboard,
                base_address_0: address,
                interrupt_line: 1,
                .. Default::default()
            },
            address,
            interrupt_queue,
            key_receiver,
            waiting: false,
        }
    }
}

impl Device for Keyboard {
    fn get_record(&self) -> Option<&DeviceRecord> {
        Some(&self.record)
    }

    fn get_memory_area(&self) -> std::ops::RangeInclusive<u32> {
        self.address..=self.address + 3
    }

    fn init_memory(&mut self, bytes: &mut Bytes) {
        bytes.write_u8(self.address, 0x01);
    }

    fn update_device(&mut self, bytes: &mut Bytes) {
        if let Ok(key) = self.key_receiver.try_recv() {
            if !self.waiting {
                self.interrupt_queue.borrow_mut().push_back((self.record.interrupt_line, None));
                self.waiting = true;
                bytes.write_u16(self.address + 2, key as u16);
                bytes.write_u8(self.address, 0x02);
            }
        }
    }

    fn write_memory(&mut self, bytes: &mut Bytes, address: u32, value: u8) -> WriteResult {
        match address - self.address {
            0 if value == 0x01 => {
                self.waiting = false;
                bytes.write_u8(self.address, 0x01);
            },
            _ => {},
        }

        WriteResult::Cancel
    }
}