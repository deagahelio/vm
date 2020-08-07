usingnamespace @import("cpu.zig");
usingnamespace @import("memory.zig");
usingnamespace @import("monitor.zig");
const std = @import("std");

pub const Vm = struct {
    cpu: Cpu,
    memory: Memory,
    monitor: *Monitor,

    pub fn init(memory_size: u32) !*Vm {
        var vm = try std.heap.c_allocator.create(Vm);

        vm.memory = Memory {
            .bytes = Bytes {
                .bytes = try std.heap.c_allocator.alloc(u8, memory_size),
            },
        };
        vm.cpu = Cpu { .memory = &vm.memory };
        vm.monitor = try Monitor.init(&vm.memory, 0x100000, 32 * 1024 * 1024);

        return vm;
    }
};