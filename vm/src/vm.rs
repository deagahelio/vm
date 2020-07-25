use crate::device::{Device, DeviceManager};
use crate::memory::{Memory, MemoryDevice};
use crate::cpu::{Exception, Cpu};
use crate::monitor::Monitor;
use crate::disk_controller::DiskController;
use crate::interrupt_controller::InterruptController;
use crate::keyboard::Keyboard;
use std::rc::Rc;
use std::cell::RefCell;
use std::thread::{spawn, JoinHandle};
use std::sync::mpsc::{channel, Receiver};
use minifb::{Window, WindowOptions, Scale, Key, KeyRepeat};

pub struct Vm {
    cpu:                      Cpu,
    pub memory:               Rc<RefCell<Memory>>,
    pub memory_device:        Rc<RefCell<MemoryDevice>>,
    pub monitor:              Rc<RefCell<Monitor>>,
    pub disk_controller:      Rc<RefCell<DiskController>>,
    pub interrupt_controller: Rc<RefCell<InterruptController>>,
    pub keyboard:             Rc<RefCell<Keyboard>>,
    pub device_manager:       Rc<RefCell<DeviceManager>>,
    devices:                  Vec<Rc<RefCell<dyn Device>>>,
}

impl Vm {
    pub fn new(memory_size: usize) -> Self {
        let memory = Rc::new(RefCell::new(Memory::new(memory_size, Vec::new())));
        let cpu = Cpu::new(memory.clone());

        let (_, key_receiver) = Self::minifb_thread(
            640, 360,
            (&memory.borrow().bytes.bytes[0x100000]) as *const _ as usize,
            32 * 1024 * 1024
        );

        let interrupt_controller = Rc::new(RefCell::new(InterruptController::new(3, 0xF2000)));
        let memory_device        = Rc::new(RefCell::new(MemoryDevice::new(0, memory_size)));
        let monitor              = Rc::new(RefCell::new(Monitor::new(1, 0x100000, 32 * 1024 * 1024)));
        let disk_controller      = Rc::new(RefCell::new(DiskController::new(2, 0xF1000)));
        let keyboard             = Rc::new(RefCell::new(Keyboard::new(4, 0xF3000, interrupt_controller.borrow_mut().queue.clone(), key_receiver)));
        let device_manager       = Rc::new(RefCell::new(DeviceManager::new(0xF0000)));

        let devices: Vec<Rc<RefCell<dyn Device>>> = vec![
            memory_device.clone(),
            monitor.clone(),
            disk_controller.clone(),
            interrupt_controller.clone(),
            keyboard.clone(),
            device_manager.clone(),
        ];

        {
            let mut device_manager = device_manager.borrow_mut();
            for device in devices.iter().take(devices.len() - 1) {
                if let Some(record) = device.borrow().get_record() {
                    device_manager.register_record(record);
                }
            }
        }

        memory.borrow_mut().devices = devices.clone();

        Self {
            cpu,
            memory,
            memory_device,
            monitor,
            disk_controller,
            interrupt_controller,
            keyboard,
            device_manager,
            devices,
        }
    }

    pub fn cycle(&mut self) -> Result<(), Exception> {
        let result = self.cpu.cycle()
            .and_then(|_| {
                {
                    let mut memory = self.memory.borrow_mut();
                    for mut device in self.devices.iter_mut().map(|dev| dev.borrow_mut()) {
                        device.update_device(&mut memory.bytes);
                    }
                }

                self.interrupt_controller.borrow_mut().send_interrupts(&mut self.cpu)
            });
        match result {
            Ok(()) =>  {
                //println!("OK ip={:X} opcode={:02X} regs={:?} sp={}", self.cpu.ip, self.memory.borrow().bytes.read_u8(self.cpu.ip).unwrap(), &self.cpu.registers[1..15], self.cpu.registers[15]);
            },
            Err(e) => {
                println!("ERROR {:?}", e);
            },
        }
        result
    }

    pub fn run(&mut self) {
        while self.cycle().is_ok() {}
    }

    pub fn minifb_thread(width: usize, height: usize, framebuffer_address: usize, framebuffer_size: usize) -> (JoinHandle<()>, Receiver<Key>) {
        let (key_sender, key_receiver) = channel();

        (
            spawn(move || {
                let mut window = Window::new("vm", 640, 360, WindowOptions { scale: Scale::X2, .. WindowOptions::default() }).unwrap();
                window.limit_update_rate(Some(std::time::Duration::from_micros(16666)));

                while window.is_open() && !window.is_key_down(Key::Escape) {
                    if let Some(keys) = window.get_keys_pressed(KeyRepeat::No) {
                        for key in keys.into_iter() {
                            key_sender.send(key).unwrap();
                        }
                    }

                    let framebuffer = unsafe {
                        std::slice::from_raw_parts(framebuffer_address as *const u32, framebuffer_size as usize)
                    };
                    window.update_with_buffer(framebuffer, width, height).unwrap();
                }
            }),
            key_receiver,
        )
    }
}