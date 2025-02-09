const std = @import("std");

const GPU = @This();

pub const screen_width = 160;
pub const screen_height = 144;

// Video RAM
vram: []u8,
// Object attribute RAM
oam: []u8,

pub fn init(alloc: std.mem.Allocator) !GPU {
    const vram = try alloc.alloc(u8, 8192);
    @memset(vram, 0);

    const oam = try alloc.alloc(u8, 8192);
    @memset(oam, 0);

    return .{ .vram = vram, .oam = oam };
}
