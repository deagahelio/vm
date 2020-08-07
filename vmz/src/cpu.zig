usingnamespace @import("memory.zig");
const std = @import("std");

pub const Exception = error {
    InvalidOpcode,
    ProtectionFault,
};

pub const Flags = packed struct {
    padding: u4 = 0,
    paging: bool = false,
    compare: bool = false,
    interrupt: bool = false,
    user_mode: bool = false,
};

pub const Cpu = struct {
    memory: *Memory,
    registers: [16]u32 = undefined,
    ip: u32 = 0x200,
    flags: Flags = Flags {},

    pub fn cycle(self: *Cpu) Exception!void {
        const opcode = self.memory.read_u8(self.ip) catch return Exception.InvalidOpcode;
        //std.debug.print("IP={}, {X:0>2}\n", .{self.ip, opcode});

        switch (opcode) {
            0x00 => { // NOP
                self.ip +%= 1;
            },
            0x01 => { // ADD a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] +%= self.registers[a];
                self.ip +%= 2;
            },
            0x02 => { // SUB a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] -%= self.registers[a];
                self.ip +%= 2;
            },
            0x03 => { // MUL a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                const result = @intCast(u64, self.registers[b]) *% self.registers[a];
                self.registers[14] = @intCast(u32, result >> 32);
                self.registers[13] = @intCast(u32, result & 0xFFFFFFFF);
                self.ip +%= 2;
            },
            0x04 => { // DIV a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[14] = self.registers[b] / self.registers[a];
                self.registers[13] = self.registers[b] % self.registers[a];
                self.ip +%= 2;
            },
            0x05 => { // AND a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] &= self.registers[a];
                self.ip +%= 2;
            },
            0x06 => { // OR a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] |= self.registers[a];
                self.ip +%= 2;
            },
            0x07 => { // XOR a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] ^= self.registers[a];
                self.ip +%= 2;
            },
            0x08 => { // SHL a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] = std.math.shl(u32, self.registers[b], self.registers[a]);
                self.ip +%= 2;
            },
            0x09 => { // SHR a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] = std.math.shr(u32, self.registers[b], self.registers[a]);
                self.ip +%= 2;
            },
            0x0A => { // STB a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                try self.memory.write_u8(self.registers[b], @intCast(u8, self.registers[a]));
                self.ip +%= 2;
            },
            0x0B => { // STW a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                try self.memory.write_u16(self.registers[b], @intCast(u16, self.registers[a]));
                self.ip +%= 2;
            },
            0x0C => { // STD a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                try self.memory.write_u32(self.registers[b], self.registers[a]);
                self.ip +%= 2;
            },
            0x0D => { // LDB a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] = try self.memory.read_u8(self.registers[a]);
                self.ip +%= 2;
            },
            0x0E => { // LDW a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] = try self.memory.read_u16(self.registers[a]);
                self.ip +%= 2;
            },
            0x0F => { // LDD a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] = try self.memory.read_u32(self.registers[a]);
                self.ip +%= 2;
            },
            0x10 => {
                const opcode2 = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = opcode2 & 0xF;
                const imm = self.memory.read_u32(self.ip + 2) catch return Exception.InvalidOpcode;

                switch (opcode2 >> 4) {
                    0x1 => { // ADDI imm a
                        self.registers[a] +%= imm;
                    },
                    0x2 => { // SUBI imm a
                        self.registers[a] -%= imm;
                    },
                    0x3 => { // MULI imm a
                        const result = @intCast(u64, imm) *% self.registers[a];
                        self.registers[14] = @intCast(u32, result >> 32);
                        self.registers[13] = @intCast(u32, result & 0xFFFFFFFF);
                    },
                    0x4 => { // DIVI imm a
                        self.registers[14] = self.registers[a] / imm;
                        self.registers[13] = self.registers[a] % imm;
                    },
                    0x5 => { // ANDI imm a
                        self.registers[a] &= imm;
                    },
                    0x6 => { // ORI imm a
                        self.registers[a] |= imm;
                    },
                    0x7 => { // XORI imm a
                        self.registers[a] ^= imm;
                    },
                    0x8 => { // SHLI imm a
                        self.registers[a] = std.math.shl(u32, self.registers[a], imm);
                    },
                    0x9 => { // SHRI imm a
                        self.registers[a] = std.math.shr(u32, self.registers[a], imm);
                    },
                    0xA => { // STBI imm a
                        try self.memory.write_u8(self.registers[a], @intCast(u8, imm));
                    },
                    0xB => { // STWI imm a
                        try self.memory.write_u16(self.registers[a], @intCast(u16, imm));
                    },
                    0xC => { // STDI imm a
                        try self.memory.write_u32(self.registers[a], imm);
                    },
                    0xD => { // LDBI imm a
                        self.registers[a] = try self.memory.read_u8(imm);
                    },
                    0xE => { // LDWI imm a
                        self.registers[a] = try self.memory.read_u16(imm);
                    },
                    0xF => { // LDDI imm a
                        self.registers[a] = try self.memory.read_u32(imm);
                    },
                    else => { return Exception.InvalidOpcode; },
                }

                self.ip +%= 6;
            },
            0x20 => {
                const opcode2 = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = opcode2 & 0xF;

                switch (opcode2 >> 4) {
                    0x1 => { // PUSH a
                        self.registers[15] -%= 4;
                        try self.memory.write_u32(self.registers[15], self.registers[a]);
                        self.ip +%= 2;
                    },
                    0x2 => { // POP a
                        self.registers[a] = try self.memory.read_u32(self.registers[15]);
                        self.registers[15] +%= 4;
                        self.ip +%= 2;
                    },
                    0x3 => { // J a
                        self.ip = self.registers[a];
                    },
                    0x4 => { // JT a
                        if (self.flags.compare) {
                            self.ip = self.registers[a];
                        } else {
                            self.ip +%= 2;
                        }
                    },
                    0x5 => { // JF a
                        if (!self.flags.compare) {
                            self.ip = self.registers[a];
                        } else {
                            self.ip +%= 2;
                        }
                    },
                    0x9 => { // CALL a
                        self.registers[15] -%= 4;
                        try self.memory.write_u32(self.registers[15], self.ip + 2);
                        self.ip = self.registers[a];
                    },
                    else => { return Exception.InvalidOpcode; },
                }
            },
            0x21 => { // PUSHI imm
                self.registers[15] -%= 4;
                const imm = self.memory.read_u32(self.ip + 1) catch return Exception.InvalidOpcode;
                try self.memory.write_u32(self.registers[15], imm);
                self.ip +%= 5;
            },
            0x23 => { // JI imm
                self.ip = self.memory.read_u32(self.ip + 1) catch return Exception.InvalidOpcode;
            },
            0x24 => { // JTI imm
                if (self.flags.compare) {
                    self.ip = self.memory.read_u32(self.ip + 1) catch return Exception.InvalidOpcode;
                } else {
                    self.ip +%= 5;
                }
            },
            0x25 => { // JFI imm
                if (!self.flags.compare) {
                    self.ip = self.memory.read_u32(self.ip + 1) catch return Exception.InvalidOpcode;
                } else {
                    self.ip +%= 5;
                }
            },
            0x29 => { // CALLI imm
                self.registers[15] -%= 4;
                try self.memory.write_u32(self.registers[15], self.ip + 5);
                self.ip = self.memory.read_u32(self.ip + 1) catch return Exception.InvalidOpcode;
            },
            0x2A => { // CGTQ a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.flags.compare = self.registers[a] >= self.registers[b];
                self.ip +%= 2;
            },
            0x2B => { // CLTQ a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.flags.compare = self.registers[a] <= self.registers[b];
                self.ip +%= 2;
            },
            0x2C => { // CEQ a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.flags.compare = self.registers[a] == self.registers[b];
                self.ip +%= 2;
            },
            0x2D => { // CNQ a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.flags.compare = self.registers[a] != self.registers[b];
                self.ip +%= 2;
            },
            0x2E => { // CGT a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.flags.compare = self.registers[a] > self.registers[b];
                self.ip +%= 2;
            },
            0x2F => { // CLT a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.flags.compare = self.registers[a] < self.registers[b];
                self.ip +%= 2;
            },
            0x30 => {
                const opcode2 = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = opcode2 & 0xF;
                const imm = self.memory.read_u32(self.ip + 2) catch return Exception.InvalidOpcode;

                switch (opcode2 >> 4) {
                    0x1 => { // MOVI imm a
                        self.registers[a] = imm;
                    },
                    0xA => { // CGTQI a imm
                        self.flags.compare = self.registers[a] >= imm;
                    },
                    0xB => { // CLTQI a imm
                        self.flags.compare = self.registers[a] <= imm;
                    },
                    0xC => { // CEQI a imm
                        self.flags.compare = self.registers[a] == imm;
                    },
                    0xD => { // CNQI a imm
                        self.flags.compare = self.registers[a] != imm;
                    },
                    0xE => { // CGTI a imm
                        self.flags.compare = self.registers[a] > imm;
                    },
                    0xF => { // CLTI a imm
                        self.flags.compare = self.registers[a] < imm;
                    },
                    else => { return Exception.InvalidOpcode; },
                }

                self.ip +%= 6;
            },
            0x31 => { // MOV a b
                const ab = self.memory.read_u8(self.ip + 1) catch return Exception.InvalidOpcode;
                const a = ab >> 4;
                const b = ab & 0xF;
                self.registers[b] = self.registers[a];
                self.ip +%= 2;
            },
            0x32 => { // STBII imm imm
                const imm1 = self.memory.read_u32(self.ip + 1) catch return Exception.InvalidOpcode;
                const imm2 = self.memory.read_u32(self.ip + 5) catch return Exception.InvalidOpcode;
                try self.memory.write_u8(imm2, @intCast(u8, imm1));
                self.ip +%= 9;
            },
            0x33 => { // STWII imm imm
                const imm1 = self.memory.read_u32(self.ip + 1) catch return Exception.InvalidOpcode;
                const imm2 = self.memory.read_u32(self.ip + 5) catch return Exception.InvalidOpcode;
                try self.memory.write_u16(imm2, @intCast(u16, imm1));
                self.ip +%= 9;
            },
            0x34 => { // STDII imm imm
                const imm1 = self.memory.read_u32(self.ip + 1) catch return Exception.InvalidOpcode;
                const imm2 = self.memory.read_u32(self.ip + 5) catch return Exception.InvalidOpcode;
                try self.memory.write_u32(imm2, imm1);
                self.ip +%= 9;
            },
            0x35 => { // RET
                self.ip = try self.memory.read_u32(self.registers[15]);
                self.registers[15] +%= 4;
            },
            0x40 => { // SYSCALL
                try self.interrupt(15, null);
            },
            0x41 => { // IRET
                const sp = self.registers[15];
                self.ip = try self.memory.read_u32(sp);
                self.registers[15] = try self.memory.read_u32(sp +% 4);
                self.flags = @bitCast(Flags, @intCast(u8, try self.memory.read_u32(sp +% 8)));
            },
            0x42 => { // CLI
                self.flags.interrupt = false;
            },
            0x43 => { // STI
                self.flags.interrupt = true;
            },
            else => { return Exception.InvalidOpcode; },
        }

        self.registers[0] = 0;
    }

    pub fn interrupt(self: *Cpu, line: u8, error_code: ?u8) Exception!void {
        if (self.flags.interrupt) {
            const sp = self.registers[15];
            try self.memory.bytes.write_u32(sp -% 4, @bitCast(u8, self.flags));
            try self.memory.bytes.write_u32(sp -% 8, sp);
            try self.memory.bytes.write_u32(sp -% 12, sp);
            try self.memory.bytes.write_u32(sp -% 16, sp);
            self.registers[15] -%= 16;

            self.ip = try self.memory.bytes.read_u32(0xF2000 + @intCast(u32, line * 4));
            self.flags.user_mode = false;
            self.flags.interrupt = false;
        }
    }
};