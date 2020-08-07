usingnamespace @import("memory.zig");
const std = @import("std");

pub const Class = enum(u8) {
    Memory = 0x1,
    DiskController = 0x2,
    InterruptController = 0x3,
    Timer = 0x4,
    PowerManager = 0x5,
    Mouse = 0x10,
    Keyboard = 0x11,
    Monitor = 0x20,
};

pub const Record = struct {
    id: u8 = 0,
    class: Class,
    interrupt_line: u8 = 0,
    base_address_0: u32 = 0,
    limit_0: u32 = 0,
    base_address_1: u32 = 0,
    limit_1: u32 = 0,
};

pub const Device = struct {
    thread: *std.Thread,
    memory: *Memory,
    record: Record,
};