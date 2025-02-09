const std = @import("std");
const GPU = @import("GPU.zig");
const bootrom = @import("./bootrom.zig").bootrom;

const MMU = @This();

gpu: GPU,
rom: []u8,

in_bios: bool = true,

// Working RAM
wram: []u8,
// Zero-page RAM
zram: []u8,
// External (cartridge) RAM
eram: []u8,

pub fn init(
    alloc: std.mem.Allocator,
    gpu: GPU,
    rom: []u8,
) !MMU {
    const wram = try alloc.alloc(u8, 8192);
    @memset(wram, 0);

    const zram = try alloc.alloc(u8, 128);
    @memset(zram, 0);

    const eram = try alloc.alloc(u8, 8192);
    @memset(eram, 0);

    return .{
        .wram = wram,
        .zram = zram,
        .eram = eram,
        .rom = rom,
        .gpu = gpu,
    };
}

fn readByte(self: *const MMU, addr: u16) u8 {
    return switch (addr) {
        // Bootrom and cartridge
        0x0000...0x7FFF => {
            if (self.in_bios) {
                if (addr < 0x100) return bootrom[addr];
                // TODO: If PC === 0x100 then set in_bios to false
            }

            return self.rom[addr];
        },

        // VRAM
        0x8000...0x9FFF => self.gpu.vram[addr & 0x1FFF],

        // External RAM
        0xA000...0xBFFF => self.eram[addr & 0x1FFF],

        // Working RAM and shadow
        0xC000...0xFDFF => self.wram[addr & 0x1FFF],

        // Graphics OAM (160 bytes)
        0xFE00...0xFE9F => self.gpu.oam[addr & 0xFF],
        // Rest of OAM is read as 0
        0xFEA0...0xFEFF => 0,

        // TODO: I/O handling
        0xFF00...0xFF7F => 0,

        // Zero-page RAM
        0xFF80...0xFFFF => self.zram[addr & 0x7F],
    };
}

pub fn read(self: *const MMU, comptime T: type, addr: u16) T {
    if (T == u8) return self.readByte(addr);
    if (T == u16) return self.readByte(addr) | (@as(u16, self.readByte(addr + 1)) << 8);
    @compileError("invalid read type" ++ @typeName(T));
}

fn writeByte(self: *const MMU, addr: u16, value: u8) void {
    switch (addr) {
        // VRAM
        0x8000...0x9FFF => self.gpu.vram[addr & 0x1FFF] = value,

        // External RAM
        0xA000...0xBFFF => self.eram[addr & 0x1FFF] = value,

        // Working RAM and shadow
        0xC000...0xFDFF => self.wram[addr & 0x1FFF] = value,

        // Graphics OAM (160 bytes)
        0xFE00...0xFE9F => self.gpu.oam[addr & 0xFF] = value,

        // TODO: I/O handling
        0xFF00...0xFF7F => {
            std.debug.print("MMU: Write to I/O location 0x{X:0>4} with value 0x{X:0>2}\n", .{ addr, value });
        },

        // Zero-page RAM
        0xFF80...0xFFFF => self.zram[addr & 0x7F] = value,

        else => {
            std.debug.print("MMU: Write to unmapped location 0x{X:0>4} with value 0x{X:0>2}\n", .{ addr, value });
        },
    }
}

pub fn write(self: *const MMU, addr: u16, value: anytype) void {
    if (@TypeOf(value) == u8) return self.writeByte(addr, value);
    if (@TypeOf(value) == u16) return self.writeWord(addr, value);
    @compileError("invalid write type " ++ @typeName(@TypeOf(value)));
}

pub fn writeWord(self: *const MMU, addr: u16, value: u16) void {
    self.writeByte(addr, @intCast(value & 0xFF));
    self.writeByte(addr + 1, @intCast(value >> 8));
}
