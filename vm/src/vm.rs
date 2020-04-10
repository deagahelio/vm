use crate::memory::Memory;
use std::fs::File;
use std::io::Read;

macro_rules! unwrap_or_return {
    ( $e:expr, $f:expr ) => {
        match $e {
            Some(x) => x,
            None => return $f,
        }
    }
}

#[derive(Debug)]
pub enum Exception {
    InvalidOpcode,
    ProtectionFault,
}

pub struct Vm {
    pub memory: Memory,
    pub registers: [u32; 16],
    pub ip: u32,
    pub cmp: bool,
}

impl Vm {
    pub fn new(memory_size: usize) -> Self {
        Self {
            memory: Memory::new(memory_size),
            registers: [0; 16],
            ip: 0,
            cmp: false,
        }
    }

    pub fn load_from_file(&mut self, file_name: &str) -> std::io::Result<()> {
        let mut file = File::open(file_name)?;
        file.read(&mut self.memory.bytes)?;
        Ok(())
    }

    pub fn cycle(&mut self) -> Result<(), Exception> {
        let opcode = unwrap_or_return!(self.memory.read_u8(self.ip), Err(Exception::ProtectionFault));
        self.ip += match opcode {
            0x00 => 1,
            0x01 => { // ADD a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] = self.registers[b as usize].wrapping_add(self.registers[a as usize]);
                2
            },
            0x02 => { // SUB a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] = self.registers[b as usize].wrapping_sub(self.registers[a as usize]);
                2
            },
            0x03 => { // MUL a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                let result = (self.registers[b as usize] as u64).wrapping_mul(self.registers[a as usize] as u64);
                self.registers[14] = (result >> 32) as u32;
                self.registers[13] = (result & 0xFFFFFFFF) as u32;
                2
            },
            0x04 => { // DIV a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[14] = self.registers[b as usize] / self.registers[a as usize];
                self.registers[13] = self.registers[b as usize] % self.registers[a as usize];
                2
            },
            0x05 => { // AND a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] &= self.registers[a as usize];
                2
            },
            0x06 => { // OR a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] |= self.registers[a as usize];
                2
            },
            0x07 => { // XOR a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] ^= self.registers[a as usize];
                2
            },
            0x08 => { // SHL a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] <<= self.registers[a as usize];
                2
            },
            0x09 => { // SHR a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] >>= self.registers[a as usize];
                2
            },
            0x0A => { // STB a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                unwrap_or_return!(self.memory.write_u8(self.registers[b as usize], self.registers[a as usize] as u8), Err(Exception::ProtectionFault));
                2
            },
            0x0B => { // STW a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                unwrap_or_return!(self.memory.write_u16(self.registers[b as usize], self.registers[a as usize] as u16), Err(Exception::ProtectionFault));
                2
            },
            0x0C => { // STD a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                unwrap_or_return!(self.memory.write_u32(self.registers[b as usize], self.registers[a as usize]), Err(Exception::ProtectionFault));
                2
            },
            0x0D => { // LDB b a
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] = unwrap_or_return!(self.memory.read_u8(self.registers[a as usize]), Err(Exception::ProtectionFault)) as u32;
                2
            },
            0x0E => { // LDW b a
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] = unwrap_or_return!(self.memory.read_u16(self.registers[a as usize]), Err(Exception::ProtectionFault)) as u32;
                2
            },
            0x0F => { // LDD b a
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] = unwrap_or_return!(self.memory.read_u32(self.registers[a as usize]), Err(Exception::ProtectionFault));
                2
            },
            0x10 => {
                let opcode2 = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let a = opcode2 & 0xF;
                let imm = unwrap_or_return!(self.memory.read_u32(self.ip + 2), Err(Exception::InvalidOpcode));
                match opcode2 >> 4 {
                    0x1 => { // ADDI imm a
                        self.registers[a as usize] = self.registers[a as usize].wrapping_add(imm);
                        6
                    },
                    0x2 => { // SUBI imm a
                        self.registers[a as usize] = self.registers[a as usize].wrapping_sub(imm);
                        6
                    },
                    0x3 => { // MULI imm a
                        let result = (self.registers[a as usize] as u64).wrapping_mul(imm as u64);
                        self.registers[14] = (result >> 32) as u32;
                        self.registers[13] = (result & 0xFFFFFFFF) as u32;
                        6
                    },
                    0x4 => { // DIVI imm a
                        self.registers[14] = self.registers[a as usize] / imm;
                        self.registers[13] = self.registers[a as usize] % imm;
                        6
                    },
                    0x5 => { // ANDI imm a
                        self.registers[a as usize] &= imm;
                        6
                    },
                    0x6 => { // ORI imm a
                        self.registers[a as usize] |= imm;
                        6
                    },
                    0x7 => { // XORI imm a
                        self.registers[a as usize] ^= imm;
                        6
                    },
                    0x8 => { // SHLI imm a
                        self.registers[a as usize] <<= imm;
                        6
                    },
                    0x9 => { // SHRI imm a
                        self.registers[a as usize] >>= imm;
                        6
                    },
                    0xA => { // STBI a imm
                        unwrap_or_return!(self.memory.write_u8(imm, self.registers[a as usize] as u8), Err(Exception::ProtectionFault));
                        6
                    },
                    0xB => { // STWI a imm
                        unwrap_or_return!(self.memory.write_u16(imm, self.registers[a as usize] as u16), Err(Exception::ProtectionFault));
                        6
                    },
                    0xC => { // STDI a imm
                        unwrap_or_return!(self.memory.write_u32(imm, self.registers[a as usize]), Err(Exception::ProtectionFault));
                        6
                    },
                    0xD => { // LDBI imm a
                        self.registers[a as usize] = unwrap_or_return!(self.memory.read_u8(imm), Err(Exception::ProtectionFault)) as u32;
                        6
                    },
                    0xE => { // LDWI imm a
                        self.registers[a as usize] = unwrap_or_return!(self.memory.read_u16(imm), Err(Exception::ProtectionFault)) as u32;
                        6
                    },
                    0xF => { // LDDI imm a
                        self.registers[a as usize] = unwrap_or_return!(self.memory.read_u32(imm), Err(Exception::ProtectionFault));
                        6
                    },
                    _ => return Err(Exception::InvalidOpcode),
                }
            },
            0x20 => {
                let opcode2 = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let a = opcode2 & 0xF;
                match opcode2 >> 4 {
                    0x1 => { // PUSH a
                        self.registers[15] = self.registers[15].wrapping_sub(4);
                        unwrap_or_return!(self.memory.write_u32(self.registers[15], self.registers[a as usize]), Err(Exception::ProtectionFault));
                        2
                    },
                    0x2 => { // POP a
                        self.registers[a as usize] = unwrap_or_return!(self.memory.read_u32(self.registers[15]), Err(Exception::ProtectionFault));
                        self.registers[15] = self.registers[15].wrapping_add(4);
                        2
                    },
                    0x3 => { // J a
                        self.ip = self.registers[a as usize];
                        0
                    },
                    0x4 => { // JT a
                        if self.cmp {
                            self.ip = self.registers[a as usize];
                            0
                        } else {
                            2
                        }
                    },
                    0x5 => { // JF a
                        if !self.cmp {
                            self.ip = self.registers[a as usize];
                            0
                        } else {
                            2
                        }
                    },
                    0x6 => { // B a
                        self.ip = (self.ip as i32).wrapping_add(self.registers[a as usize] as i32) as u32;
                        0
                    },
                    0x7 => { // BT a
                        if self.cmp {
                            self.ip = (self.ip as i32).wrapping_add(self.registers[a as usize] as i32) as u32;
                            0
                        } else {
                            2
                        }
                    },
                    0x8 => { // BF a
                        if !self.cmp {
                            self.ip = (self.ip as i32).wrapping_add(self.registers[a as usize] as i32) as u32;
                            0
                        } else {
                            2
                        }
                    },
                    0x9 => { // CALL a
                        self.registers[15] = self.registers[15].wrapping_sub(4);
                        unwrap_or_return!(self.memory.write_u32(self.registers[15], self.ip + 2), Err(Exception::ProtectionFault));
                        self.ip = self.registers[a as usize];
                        0
                    },
                    _ => return Err(Exception::InvalidOpcode),
                }
            },
            0x21 => { // PUSHI imm
                self.registers[15] = self.registers[15].wrapping_sub(4);
                unwrap_or_return!(self.memory.write_u32(
                    self.registers[15],
                    unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode))
                ), Err(Exception::ProtectionFault));
                5
            },
            0x23 => { // JI imm
                self.ip = unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode));
                0
            },
            0x24 => { // JTI imm
                if self.cmp {
                    self.ip = unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode));
                    0
                } else {
                    5
                }
            },
            0x25 => { // JFI imm
                if !self.cmp {
                    self.ip = unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode));
                    0
                } else {
                    5
                }
            },
            0x26 => { // BI imm
                self.ip = (self.ip as i32).wrapping_add(unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode)) as i32) as u32;
                0
            },
            0x27 => { // BTI imm
                if self.cmp {
                    self.ip = (self.ip as i32).wrapping_add(unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode)) as i32) as u32;
                    0
                } else {
                    5
                }
            },
            0x28 => { // BFI imm
                if !self.cmp {
                    self.ip = (self.ip as i32).wrapping_add(unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode)) as i32) as u32;
                    0
                } else {
                    5
                }
            },
            0x29 => { // CALLI imm
                self.registers[15] = self.registers[15].wrapping_sub(4);
                unwrap_or_return!(self.memory.write_u32(self.registers[15], self.ip + 5), Err(Exception::ProtectionFault));
                self.ip = unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode));
                0
            },
            0x2C => { // CEQ a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.cmp = self.registers[a as usize] == self.registers[b as usize];
                2
            },
            0x2D => { // CNQ a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.cmp = self.registers[a as usize] == self.registers[b as usize];
                2
            },
            0x2E => { // CGT a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.cmp = self.registers[a as usize] > self.registers[b as usize];
                2
            },
            0x2F => { // CLT a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.cmp = self.registers[a as usize] < self.registers[b as usize];
                2
            },
            0x30 => {
                let opcode2 = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let a = opcode2 & 0xF;
                let imm = unwrap_or_return!(self.memory.read_u32(self.ip + 2), Err(Exception::InvalidOpcode));
                match opcode2 >> 4 {
                    0x1 => { // MOVI imm a
                        self.registers[a as usize] = imm;
                        6
                    },
                    0xC => { // CEQI a imm
                        self.cmp = self.registers[a as usize] == imm;
                        6
                    },
                    0xD => { // CNQI a imm
                        self.cmp = self.registers[a as usize] != imm;
                        6
                    },
                    0xE => { // CGTI a imm
                        self.cmp = self.registers[a as usize] > imm;
                        6
                    },
                    0xF => { // CLTI a imm
                        self.cmp = self.registers[a as usize] < imm;
                        6
                    },
                    _ => return Err(Exception::InvalidOpcode),
                }
            },
            0x31 => { // MOV a b
                let ab = unwrap_or_return!(self.memory.read_u8(self.ip + 1), Err(Exception::InvalidOpcode));
                let (a, b) = (ab >> 4, ab & 0xF);
                self.registers[b as usize] = self.registers[a as usize];
                2
            },
            0x32 => { // STBII imm imm
                let imm1 = unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode));
                let imm2 = unwrap_or_return!(self.memory.read_u32(self.ip + 5), Err(Exception::InvalidOpcode));
                unwrap_or_return!(self.memory.write_u8(imm2, imm1 as u8), Err(Exception::ProtectionFault));
                9
            },
            0x33 => { // STWII imm imm
                let imm1 = unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode));
                let imm2 = unwrap_or_return!(self.memory.read_u32(self.ip + 5), Err(Exception::InvalidOpcode));
                unwrap_or_return!(self.memory.write_u16(imm2, imm1 as u16), Err(Exception::ProtectionFault));
                9
            },
            0x34 => { // STDII imm imm
                let imm1 = unwrap_or_return!(self.memory.read_u32(self.ip + 1), Err(Exception::InvalidOpcode));
                let imm2 = unwrap_or_return!(self.memory.read_u32(self.ip + 5), Err(Exception::InvalidOpcode));
                unwrap_or_return!(self.memory.write_u32(imm2, imm1), Err(Exception::ProtectionFault));
                9
            },
            0x35 => { // RET
                self.ip = unwrap_or_return!(self.memory.read_u32(self.registers[15]), Err(Exception::ProtectionFault));
                self.registers[15] = self.registers[15].wrapping_add(4);
                0
            },
            _ => return Err(Exception::InvalidOpcode),
        };

        self.registers[0] = 0;
        Ok(())
    }
}