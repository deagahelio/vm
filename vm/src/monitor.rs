use crate::device::{Class, DeviceRecord, Device, WriteResult};
use crate::memory::Bytes;
use minifb::{Window, WindowOptions, Scale, Key};
use std::sync::mpsc::{channel, Sender};
use std::thread::{spawn, JoinHandle};

pub struct Monitor {
    record: DeviceRecord,
    address: u32,
    framebuffer_address: u32,
    framebuffer_size: u32,
    render_thread: JoinHandle<()>,
    render_thread_sender: Sender<usize>,
}

impl Monitor {
    pub fn new(id: u8, address: u32, framebuffer_size: u32, width: usize, height: usize) -> Self {
        let (sender, receiver) = channel();

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
            render_thread: spawn(move || {
                let mut window = Window::new("vm", width, height, WindowOptions { scale: Scale::X2, .. WindowOptions::default() }).unwrap();
                window.limit_update_rate(Some(std::time::Duration::from_micros(16666)));

                while window.is_open() && !window.is_key_down(Key::Escape) {
                    if let Ok(address) = receiver.try_recv() {
                        let framebuffer: &[u32];
                        unsafe {
                            framebuffer = std::slice::from_raw_parts(address as *const u32, framebuffer_size as usize);
                        }
                        window.update_with_buffer(framebuffer, width, height).unwrap();
                    }
                }
            }),
            render_thread_sender: sender,
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

    fn write_memory(&mut self, bytes: &mut Bytes, _address: u32, _value: u8) -> WriteResult {
        self.render_thread_sender.send((&bytes.bytes[self.framebuffer_address as usize]) as *const _ as usize).unwrap();
        WriteResult::Write
    }
}