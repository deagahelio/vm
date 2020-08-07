usingnamespace @import("cpu.zig");
const std = @import("std");

pub const Bytes = struct {
    bytes: []u8,

    pub fn read_u8(self: *Bytes, address: u32) !u8 {
        return if (address < self.bytes.len)
            self.bytes[address]
        else
            Exception.ProtectionFault;
    }

    pub fn read_u16(self: *Bytes, address: u32) !u16 {
        return if (address < self.bytes.len - 1)
            std.mem.readIntLittle(u16, @ptrCast(*[2]u8, self.bytes[address..address+2]))
        else
            Exception.ProtectionFault;
    }

    pub fn read_u32(self: *Bytes, address: u32) !u32 {
        return if (address < self.bytes.len - 3)
            std.mem.readIntLittle(u32, @ptrCast(*[4]u8, self.bytes[address..address+4]))
        else
            Exception.ProtectionFault;
    }

    pub fn write_u8(self: *Bytes, address: u32, value: u8) !void {
        if (address < self.bytes.len) {
            self.bytes[address] = value;
        } else {
            return Exception.ProtectionFault;
        }
    }

    pub fn write_u16(self: *Bytes, address: u32, value: u16) !void {
        if (address < self.bytes.len - 1) {
            std.mem.writeIntLittle(u16, @ptrCast(*[2]u8, self.bytes[address..address+2]), value);
        } else {
            return Exception.ProtectionFault;
        }
    }

    pub fn write_u32(self: *Bytes, address: u32, value: u32) !void {
        if (address < self.bytes.len - 3) {
            std.mem.writeIntLittle(u32, @ptrCast(*[4]u8, self.bytes[address..address+4]), value);
        } else {
            return Exception.ProtectionFault;
        }
    }
};

pub const Memory = struct {
    bytes: Bytes,

    pub fn load_boot(self: *Memory, file_name: []const u8) !void {
        const buffer = try std.fs.cwd().readFileAlloc(std.heap.c_allocator, file_name, std.math.maxInt(usize));
        std.mem.copy(u8, self.bytes.bytes[0x200..], buffer);
        //std.debug.print("{X:0>2}...\n", .{self.bytes.bytes[0x200..0x300]});
    }

    pub fn read_u8(self: *Memory, address: u32) !u8 {
        return self.bytes.read_u8(address);
    }

    pub fn read_u16(self: *Memory, address: u32) !u16 {
        return self.bytes.read_u16(address);
    }

    pub fn read_u32(self: *Memory, address: u32) !u32 {
        return self.bytes.read_u32(address);
    }

    pub fn write_u8(self: *Memory, address: u32, value: u8) !void {
        return self.bytes.write_u8(address, value);
    }

    pub fn write_u16(self: *Memory, address: u32, value: u16) !void {
        return self.bytes.write_u16(address, value);
    }

    pub fn write_u32(self: *Memory, address: u32, value: u32) !void {
        return self.bytes.write_u32(address, value);
    }
};