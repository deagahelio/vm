usingnamespace @import("cpu.zig");
usingnamespace @import("memory.zig");
usingnamespace @import("vm.zig");
const std = @import("std");

pub fn main() !void {
    var vm = try Vm.init(128 * 1024 * 1024);

    var args = std.process.args();
    var flag_b = false;

    _ = args.skip();

    while (args.next(std.heap.c_allocator)) |arg_maybe| {
        const arg = try arg_maybe;
        defer std.heap.c_allocator.free(arg);

        if (std.mem.startsWith(u8, arg, "-b")) {
            if (!flag_b) {
                std.debug.print("Loading boot firmware from file '{}'\n", .{arg[2..]});
                try vm.memory.load_boot(arg[2..]);
                flag_b = true;
            } else {
                std.debug.print("Ignoring '{}', boot firmware already loaded\n", .{arg[2..]});
            }
        } else {
            std.debug.print("Invalid argument '{}'\n", .{arg});
        }
    }

    var timer = try std.time.Timer.start();
    var ins: u64 = 0;
    var a = false;
    while (true) {
        _ = try vm.cpu.cycle();
        ins += 1;
        if (timer.read() >= 1000000000) {
            std.debug.print("CPU Instructions per second: {}\n", .{ins});
            ins = 0;
            timer.reset();
        }
    }
}