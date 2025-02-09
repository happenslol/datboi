const std = @import("std");
const GPU = @import("GPU.zig");
const MMU = @import("MMU.zig");
const CPU = @import("CPU.zig");

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const alloc = arena.allocator();

    // if (std.os.argv.len != 2) {
    //     std.debug.print("Usage: datboi <rom>\n", .{});
    //     return;
    // }
    //
    // const rom_path = std.mem.span(std.os.argv[1]);
    // const rom_file = try std.fs.cwd().openFile(rom_path, .{});
    // defer rom_file.close();
    //
    // const rom_stat = try rom_file.stat();
    // const rom = try rom_file.readToEndAlloc(alloc, rom_stat.size);

    const rom = try alloc.alloc(u8, 8192);
    @memset(rom, 0);

    const gpu = try GPU.init(alloc);
    var mmu = try MMU.init(alloc, gpu, rom);
    var cpu = CPU.init(&mmu);

    for (0..200) |_| {
        cpu.step();
    }
}
