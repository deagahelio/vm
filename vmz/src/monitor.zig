usingnamespace @cImport({
    @cInclude("MiniFB.h");
});
usingnamespace @import("memory.zig");
usingnamespace @import("device.zig");
const std = @import("std");

pub const Monitor = struct {
    device: Device,

    fn thread(device: *Device) void {
        var window: ?*mfb_window = mfb_open("vmz", 640, 360);
        if (window == null) {
            std.debug.print("Could not create window\n", .{});
            return;
        }

        mfb_set_target_fps(60);

        while (true) {
            const state = mfb_update(window, @ptrCast(*c_void, device.memory.bytes.bytes[device.record.base_address_1..]));

            if (state != mfb_update_state.STATE_OK or !mfb_wait_sync(window)) {
                break;
            }
        }
    }

    pub fn init(memory: *Memory, address: u32, framebuffer_size: u32) !*Monitor {
        var monitor = try std.heap.c_allocator.create(Monitor);

        monitor.* = Monitor {
            .device = Device {
                .thread = undefined,
                .memory = memory,
                .record = Record {
                    .class = Class.Monitor,
                    .base_address_0 = address + framebuffer_size,
                    .base_address_1 = address,
                    .limit_1 = address + framebuffer_size - 1,
                },
            },
        };
        monitor.device.thread = try std.Thread.spawn(&monitor.device, Monitor.thread);

        return monitor;
    }
};