#[derive(Clone)]
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
pub struct Device {
    pub id: u8,
    pub class: Class,
    pub interrupt_line: u8,
    pub base_address_0: u32,
    pub limit_0: u32,
    pub base_address_1: u32,
    pub limit_1: u32,
}