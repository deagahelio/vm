use crate::device::{Device, DeviceManager};
use crate::memory::{Memory, MemoryDevice};
use crate::cpu::{Exception, Cpu};
use crate::monitor::Monitor;
use crate::disk_controller::DiskController;
use crate::interrupt_controller::InterruptController;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Vm {
    cpu: Cpu,
    pub memory: Memory,
    pub memory_device: Rc<RefCell<MemoryDevice>>,
    pub monitor: Rc<RefCell<Monitor>>,
    pub disk_controller: Rc<RefCell<DiskController>>,
    pub interrupt_controller: Rc<RefCell<InterruptController>>,
    pub device_manager: Rc<RefCell<DeviceManager>>,
}

impl Vm {
    pub fn new(memory_size: usize) -> Self {
        let memory_device        = Rc::new(RefCell::new(MemoryDevice::new(0, memory_size)));
        let monitor              = Rc::new(RefCell::new(Monitor::new(1, 0x100000, 32 * 1024 * 1024, 640, 360)));
        let disk_controller      = Rc::new(RefCell::new(DiskController::new(2, 0xF1000)));
        let interrupt_controller = Rc::new(RefCell::new(InterruptController::new(3, 0xF2000)));
        let device_manager       = Rc::new(RefCell::new(DeviceManager::new(0xF0000)));

        let devices: Vec<Rc<RefCell<dyn Device>>> = vec![
            memory_device.clone(),
            monitor.clone(),
            disk_controller.clone(),
            interrupt_controller.clone(),
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

        Self {
            cpu: Cpu::new(),
            memory: Memory::new(memory_size, devices),
            memory_device,
            monitor,
            disk_controller,
            interrupt_controller,
            device_manager,
        }
    }

    pub fn cycle(&mut self) -> Result<(), Exception> {
        let result = self.cpu.cycle(&mut self.memory);
        match result {
            Ok(()) =>  {
                //println!("OK ip={:X} opcode={:02X} regs={:?} sp={}", vm.ip, vm.memory.read_u8(vm.ip).unwrap(), &vm.registers[1..15], vm.registers[15]);
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
}