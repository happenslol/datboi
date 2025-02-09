const std = @import("std");
const MMU = @import("MMU.zig");

const CPU = @This();

clock: u32 = 0,

regs: packed struct {
    a: u8 = 0,
    f: packed struct {
        zero: bool = false,
        subtraction: bool = false,
        half_carry: bool = false,
        carry: bool = false,

        _padding: u4 = 0,
    } = .{},

    b: u8 = 0,
    c: u8 = 0,

    d: u8 = 0,
    e: u8 = 0,

    h: u8 = 0,
    l: u8 = 0,

    pc: u16 = 0,
    sp: u16 = 0,
} = .{},

mmu: *MMU,

pub fn init(mmu: *MMU) CPU {
    return .{ .mmu = mmu };
}

const Operand = enum {
    A,
    B,
    C,
    D,
    E,
    H,
    L,

    AF,
    BC,
    DE,
    HL,
    SP,

    @"(C)",

    @"(BC)",
    @"(DE)",
    @"(HL)",
    @"(SP)",

    @"(HL+)",
    @"(HL-)",

    s8,
    d8,
    d16,
    a16,
    @"(a8)",
    @"(a16)",

    @"(SP)+s8",
};

const Condition = enum { None, Z, NZ, C, NC };

fn OperandType(comptime r: Operand) type {
    return switch (r) {
        .A, .B, .C, .D, .E, .H, .L, .d8 => u8,
        .@"(HL)" => u8,
        .BC, .DE, .HL, .SP => u16,
        else => @compileError("operand has no type: " ++ @tagName(r)),
    };
}

fn ptr(self: *CPU, comptime r: Operand) *OperandType(r) {
    return switch (r) {
        .A => &self.regs.a,
        .B => &self.regs.b,
        .C => &self.regs.c,
        .D => &self.regs.d,
        .E => &self.regs.e,
        .H => &self.regs.h,
        .L => &self.regs.l,

        .BC => @ptrCast(&self.regs.b),
        .DE => @ptrCast(&self.regs.d),
        .HL => @ptrCast(&self.regs.h),

        .SP => &self.regs.sp,

        else => @compileError("cannot get pointer to operand " ++ @tagName(r)),
    };
}

fn val(self: *CPU, comptime r: Operand) OperandType(r) {
    return switch (r) {
        .A, .B, .C, .D, .E, .H, .L => self.ptr(r).*,
        .BC, .DE, .HL, .SP => self.ptr(r).*,

        .@"(HL)" => self.mmu.read(OperandType(r), self.ptr(.HL).*),
        .d8 => self.fetch(u8),

        else => @compileError("cannot get operand value for " ++ @tagName(r)),
    };
}

fn set(self: *CPU, comptime r: Operand, value: anytype) void {
    switch (r) {
        .A, .B, .C, .D, .E, .H, .L => self.ptr(r).* = value,
        .BC, .DE, .HL, .SP => self.ptr(r).* = value,
        .@"(HL)" => self.mmu.write(self.ptr(.HL).*, value),

        else => @compileError("cannot set operand value for " ++ @tagName(r)),
    }
}

pub fn fetch(self: *CPU, comptime T: type) T {
    if (T == u8) {
        const result = self.mmu.read(T, self.regs.pc);
        self.regs.pc += 1;
        return result;
    }

    if (T == u16) {
        const result = self.mmu.read(T, self.regs.pc);
        self.regs.pc += 2;
        return result;
    }

    if (T == i8) {
        const result = @as(i8, @bitCast(self.mmu.read(u8, self.regs.pc)));
        self.regs.pc += 1;
        return result;
    }

    @compileError("invalid fetch type" ++ @typeName(T));
}

pub fn step(self: *CPU) void {
    const opcode = self.fetch(u8);

    self.clock += switch (opcode) {
        0x00 => self.NOP(1),
        0x01 => self.LD(.BC, .d16, 3),
        0x02 => self.LD(.@"(BC)", .A, 2),
        0x03 => self.INC(.BC, 2),
        0x04 => self.INC(.B, 1),
        0x05 => self.DEC(.B, 1),
        0x06 => self.LD(.B, .d8, 2),
        0x07 => self.RLCA(1),
        0x08 => self.LD(.@"(a16)", .SP, 5),
        0x09 => self.ADD(.HL, .BC, 2),
        0x0A => self.LD(.A, .@"(BC)", 2),
        0x0B => self.DEC(.BC, 2),
        0x0C => self.INC(.C, 1),
        0x0D => self.DEC(.C, 1),
        0x0E => self.LD(.C, .d8, 2),
        0x0F => self.RRCA(1),
        0x10 => self.STOP(1),
        0x11 => self.LD(.DE, .d16, 3),
        0x12 => self.LD(.@"(DE)", .A, 2),
        0x13 => self.INC(.DE, 2),
        0x14 => self.INC(.D, 1),
        0x15 => self.DEC(.D, 1),
        0x16 => self.LD(.D, .d8, 2),
        0x17 => self.RLA(1),
        0x18 => self.JR(.None, 3),
        0x19 => self.ADD(.HL, .DE, 2),
        0x1A => self.LD(.A, .@"(DE)", 2),
        0x1B => self.DEC(.DE, 2),
        0x1C => self.INC(.E, 1),
        0x1D => self.DEC(.E, 1),
        0x1E => self.LD(.E, .d8, 2),
        0x1F => self.RRA(1),
        0x20 => self.JR(.NZ, .{ .nz = 3, .z = 2 }),
        0x21 => self.LD(.HL, .d16, 3),
        0x22 => self.LD(.@"(HL+)", .A, 2),
        0x23 => self.INC(.HL, 2),
        0x24 => self.INC(.H, 1),
        0x25 => self.DEC(.H, 1),
        0x26 => self.LD(.H, .d8, 2),
        0x27 => self.DAA(1),
        0x28 => self.JR(.Z, .{ .nz = 3, .z = 2 }),
        0x29 => self.ADD(.HL, .HL, 2),
        0x2A => self.LD(.A, .@"(HL+)", 2),
        0x2B => self.DEC(.HL, 2),
        0x2C => self.INC(.L, 1),
        0x2D => self.DEC(.L, 1),
        0x2E => self.LD(.L, .d8, 2),
        0x2F => self.CPL(1),
        0x30 => self.JR(.NC, .{ .nc = 3, .c = 2 }),
        0x31 => self.LD(.SP, .d16, 3),
        0x32 => self.LD(.@"(HL-)", .A, 2),
        0x33 => self.INC(.SP, 2),
        0x34 => self.INC(.@"(HL)", 3),
        0x35 => self.DEC(.@"(HL)", 3),
        0x36 => self.LD(.@"(HL)", .d8, 3),
        0x37 => self.SCF(1),
        0x38 => self.JR(.C, .{ .nc = 3, .c = 2 }),
        0x39 => self.ADD(.HL, .SP, 2),
        0x3A => self.LD(.A, .@"(HL-)", 2),
        0x3B => self.DEC(.SP, 2),
        0x3C => self.INC(.A, 1),
        0x3D => self.DEC(.A, 1),
        0x3E => self.LD(.A, .d8, 2),
        0x3F => self.CCF(1),
        0x40 => self.LD(.B, .B, 1),
        0x41 => self.LD(.B, .C, 1),
        0x42 => self.LD(.B, .D, 1),
        0x43 => self.LD(.B, .E, 1),
        0x44 => self.LD(.B, .H, 1),
        0x45 => self.LD(.B, .L, 1),
        0x46 => self.LD(.B, .@"(HL)", 2),
        0x47 => self.LD(.B, .A, 1),
        0x48 => self.LD(.C, .B, 1),
        0x49 => self.LD(.C, .C, 1),
        0x4A => self.LD(.C, .D, 1),
        0x4B => self.LD(.C, .E, 1),
        0x4C => self.LD(.C, .H, 1),
        0x4D => self.LD(.C, .L, 1),
        0x4E => self.LD(.C, .@"(HL)", 2),
        0x4F => self.LD(.C, .A, 1),
        0x50 => self.LD(.D, .B, 1),
        0x51 => self.LD(.D, .C, 1),
        0x52 => self.LD(.D, .D, 1),
        0x53 => self.LD(.D, .E, 1),
        0x54 => self.LD(.D, .H, 1),
        0x55 => self.LD(.D, .L, 1),
        0x56 => self.LD(.D, .@"(HL)", 2),
        0x57 => self.LD(.D, .A, 1),
        0x58 => self.LD(.E, .B, 1),
        0x59 => self.LD(.E, .C, 1),
        0x5A => self.LD(.E, .D, 1),
        0x5B => self.LD(.E, .E, 1),
        0x5C => self.LD(.E, .H, 1),
        0x5D => self.LD(.E, .L, 1),
        0x5E => self.LD(.E, .@"(HL)", 2),
        0x5F => self.LD(.E, .A, 1),
        0x60 => self.LD(.H, .B, 1),
        0x61 => self.LD(.H, .C, 1),
        0x62 => self.LD(.H, .D, 1),
        0x63 => self.LD(.H, .E, 1),
        0x64 => self.LD(.H, .H, 1),
        0x65 => self.LD(.H, .L, 1),
        0x66 => self.LD(.H, .@"(HL)", 2),
        0x67 => self.LD(.H, .A, 1),
        0x68 => self.LD(.L, .B, 1),
        0x69 => self.LD(.L, .C, 1),
        0x6A => self.LD(.L, .D, 1),
        0x6B => self.LD(.L, .E, 1),
        0x6C => self.LD(.L, .H, 1),
        0x6D => self.LD(.L, .L, 1),
        0x6E => self.LD(.L, .@"(HL)", 2),
        0x6F => self.LD(.L, .A, 1),
        0x70 => self.LD(.@"(HL)", .B, 2),
        0x71 => self.LD(.@"(HL)", .C, 2),
        0x72 => self.LD(.@"(HL)", .D, 2),
        0x73 => self.LD(.@"(HL)", .E, 2),
        0x74 => self.LD(.@"(HL)", .H, 2),
        0x75 => self.LD(.@"(HL)", .L, 2),
        0x76 => self.HALT(1),
        0x77 => self.LD(.@"(HL)", .A, 2),
        0x78 => self.LD(.A, .B, 1),
        0x79 => self.LD(.A, .C, 1),
        0x7A => self.LD(.A, .D, 1),
        0x7B => self.LD(.A, .E, 1),
        0x7C => self.LD(.A, .H, 1),
        0x7D => self.LD(.A, .L, 1),
        0x7E => self.LD(.A, .@"(HL)", 2),
        0x7F => self.LD(.A, .A, 1),
        0x80 => self.ADD(.A, .B, 1),
        0x81 => self.ADD(.A, .C, 1),
        0x82 => self.ADD(.A, .D, 1),
        0x83 => self.ADD(.A, .E, 1),
        0x84 => self.ADD(.A, .H, 1),
        0x85 => self.ADD(.A, .L, 1),
        0x86 => self.ADD(.A, .@"(HL)", 2),
        0x87 => self.ADD(.A, .A, 1),
        0x88 => self.ADC(.A, .B, 1),
        0x89 => self.ADC(.A, .C, 1),
        0x8A => self.ADC(.A, .D, 1),
        0x8B => self.ADC(.A, .E, 1),
        0x8C => self.ADC(.A, .H, 1),
        0x8D => self.ADC(.A, .L, 1),
        0x8E => self.ADC(.A, .@"(HL)", 2),
        0x8F => self.ADC(.A, .A, 1),
        0x90 => self.SUB(.B, 1),
        0x91 => self.SUB(.C, 1),
        0x92 => self.SUB(.D, 1),
        0x93 => self.SUB(.E, 1),
        0x94 => self.SUB(.H, 1),
        0x95 => self.SUB(.L, 1),
        0x96 => self.SUB(.@"(HL)", 2),
        0x97 => self.SUB(.A, 1),
        0x98 => self.SBC(.A, .B, 1),
        0x99 => self.SBC(.A, .C, 1),
        0x9A => self.SBC(.A, .D, 1),
        0x9B => self.SBC(.A, .E, 1),
        0x9C => self.SBC(.A, .H, 1),
        0x9D => self.SBC(.A, .L, 1),
        0x9E => self.SBC(.A, .@"(HL)", 2),
        0x9F => self.SBC(.A, .A, 1),
        0xA0 => self.AND(.B, 1),
        0xA1 => self.AND(.C, 1),
        0xA2 => self.AND(.D, 1),
        0xA3 => self.AND(.E, 1),
        0xA4 => self.AND(.H, 1),
        0xA5 => self.AND(.L, 1),
        0xA6 => self.AND(.@"(HL)", 2),
        0xA7 => self.AND(.A, 1),
        0xA8 => self.XOR(.B, 1),
        0xA9 => self.XOR(.C, 1),
        0xAA => self.XOR(.D, 1),
        0xAB => self.XOR(.E, 1),
        0xAC => self.XOR(.H, 1),
        0xAD => self.XOR(.L, 1),
        0xAE => self.XOR(.@"(HL)", 2),
        0xAF => self.XOR(.A, 1),
        0xB0 => self.OR(.B, 1),
        0xB1 => self.OR(.C, 1),
        0xB2 => self.OR(.D, 1),
        0xB3 => self.OR(.E, 1),
        0xB4 => self.OR(.H, 1),
        0xB5 => self.OR(.L, 1),
        0xB6 => self.OR(.@"(HL)", 2),
        0xB7 => self.OR(.A, 1),
        0xB8 => self.CP(.B, 1),
        0xB9 => self.CP(.C, 1),
        0xBA => self.CP(.D, 1),
        0xBB => self.CP(.E, 1),
        0xBC => self.CP(.H, 1),
        0xBD => self.CP(.L, 1),
        0xBE => self.CP(.@"(HL)", 2),
        0xBF => self.CP(.A, 1),
        0xC0 => self.RET(.NZ, .{ .nz = 5, .z = 2 }),
        0xC1 => self.POP(.BC, 3),
        0xC2 => self.JP(.NZ, .a16, .{ .nz = 4, .z = 3 }),
        0xC3 => self.JP(.None, .a16, 4),
        0xC4 => self.CALL(.NZ, .a16, .{ .nz = 6, .z = 3 }),
        0xC5 => self.PUSH(.BC, 4),
        0xC6 => self.ADD(.A, .d8, 2),
        0xC7 => self.RST(0, 4),
        0xC8 => self.RET(.Z, .{ .z = 5, .nz = 2 }),
        0xC9 => self.RET(.None, 4),
        0xCA => self.JP(.Z, .a16, .{ .z = 4, .nz = 3 }),
        0xCC => self.CALL(.Z, .a16, .{ .z = 6, .nz = 3 }),
        0xCD => self.CALL(.None, .a16, 6),
        0xCE => self.ADC(.A, .d8, 2),
        0xCF => self.RST(1, 4),
        0xD0 => self.RET(.NC, .{ .nc = 5, .c = 2 }),
        0xD1 => self.POP(.DE, 3),
        0xD2 => self.JP(.NC, .a16, .{ .nc = 4, .c = 3 }),
        0xD4 => self.CALL(.NC, .a16, .{ .nc = 6, .c = 3 }),
        0xD5 => self.PUSH(.DE, 4),
        0xD6 => self.SUB(.d8, 2),
        0xD7 => self.RST(2, 4),
        0xD8 => self.RET(.C, .{ .c = 5, .nc = 2 }),
        0xD9 => self.RETI(4),
        0xDA => self.JP(.C, .a16, .{ .c = 4, .nc = 3 }),
        0xDC => self.CALL(.C, .a16, .{ .c = 6, .nc = 3 }),
        0xDE => self.SBC(.A, .d8, 2),
        0xDF => self.RST(3, 4),
        0xE0 => self.LD(.@"(a8)", .A, 3),
        0xE1 => self.POP(.HL, 3),
        0xE2 => self.LD(.@"(C)", .A, 2),
        0xE5 => self.PUSH(.HL, 4),
        0xE6 => self.AND(.d8, 2),
        0xE7 => self.RST(4, 4),
        0xE8 => self.ADD(.SP, .s8, 4),
        0xE9 => self.JP(.None, .HL, 1),
        0xEA => self.LD(.@"(a16)", .A, 4),
        0xEE => self.XOR(.d8, 2),
        0xEF => self.RST(5, 4),
        0xF0 => self.LD(.A, .@"(a8)", 3),
        0xF1 => self.POP(.AF, 3),
        0xF2 => self.LD(.A, .@"(C)", 2),
        0xF3 => self.DI(1),
        0xF5 => self.PUSH(.AF, 4),
        0xF6 => self.OR(.d8, 2),
        0xF7 => self.RST(6, 4),
        0xF8 => self.LD(.HL, .@"(SP)+s8", 3),
        0xF9 => self.LD(.SP, .HL, 2),
        0xFA => self.LD(.A, .@"(a16)", 4),
        0xFB => self.EI(1),
        0xFE => self.CP(.d8, 2),
        0xFF => self.RST(7, 4),

        0xCB => self.CB(),

        else => unknown: {
            std.debug.print("Unknown opcode: 0x{X:0>2}\n", .{opcode});
            break :unknown 0;
        },
    };
}

fn NOP(_: *CPU, comptime cycles: u16) u16 {
    return cycles;
}

fn LD(self: *CPU, comptime dst: Operand, comptime src: Operand, comptime cycles: u16) u16 {
    const value = switch (src) {
        .A, .B, .C, .D, .E, .H, .L, .BC, .DE, .HL, .SP => self.ptr(src).*,
        .@"(C)" => self.mmu.read(OperandType(dst), @as(u16, @intCast(self.ptr(.C).*))),
        .@"(BC)" => self.mmu.read(OperandType(dst), self.ptr(.BC).*),
        .@"(DE)" => self.mmu.read(OperandType(dst), self.ptr(.DE).*),
        .@"(HL)" => self.mmu.read(OperandType(dst), self.ptr(.HL).*),
        .d8 => self.fetch(u8),
        .d16 => self.fetch(u16),
        .s8 => self.fetch(i8),

        .@"(HL+)" => 0, // TODO
        .@"(HL-)" => 0, // TODO

        .@"(a8)" => 0, // TODO
        .@"(a16)" => 0, // TODO

        .@"(SP)+s8" => 0, // TODO

        else => @compileError("invalid LD source operand " ++ @tagName(src)),
    };

    switch (dst) {
        .A, .B, .C, .D, .E, .H, .L, .BC, .DE, .HL, .SP => self.ptr(dst).* = value,
        .@"(C)" => self.mmu.write(@as(u16, @intCast(self.ptr(.C).*)), value),
        .@"(BC)" => self.mmu.write(self.ptr(.BC).*, value),
        .@"(DE)" => self.mmu.write(self.ptr(.DE).*, value),
        .@"(HL)" => self.mmu.write(self.ptr(.HL).*, value),
        .@"(HL+)" => {}, // TODO
        .@"(HL-)" => {}, // TODO
        .@"(a8)" => self.mmu.write(self.fetch(u8), value),
        .@"(a16)" => self.mmu.write(self.fetch(u16), value),
        else => @compileError("invalid LD destination operand " ++ @tagName(dst)),
    }

    return cycles;
}

fn INC(self: *CPU, comptime a: Operand, comptime cycles: u16) u16 {
    const result = self.val(a) +% 1;
    self.set(a, result);

    self.regs.f = .{
        .zero = result == 0,
        .carry = self.regs.f.carry,
        .half_carry = (result & 0x0F) + 1 > 0x0F,
        .subtraction = false,
    };

    return cycles;
}

fn DEC(self: *CPU, comptime a: Operand, comptime cycles: u32) u32 {
    const result = self.val(a) -% 1;
    self.set(a, result);

    self.regs.f = .{
        .zero = result == 0,
        .carry = self.regs.f.carry,
        .half_carry = (result & 0x0F) == 0,
        .subtraction = true,
    };

    return cycles;
}

fn SBC(self: *CPU, comptime a: Operand, comptime b: Operand, comptime cycles: u32) u32 {
    const c: u8 = if (self.regs.f.carry) 1 else 0;
    const a_ptr = self.ptr(a);
    const a_val = a_ptr.*;
    const b_val = self.val(b);

    const result = a_val -% b_val -% c;
    a_ptr.* = result;

    self.regs.f = .{
        .zero = result == 0,
        .subtraction = true,
        .half_carry = (result & 0xF) < (result & 0xF) + c,
        .carry = result < b_val + c,
    };

    return cycles;
}

fn XOR(self: *CPU, comptime b: Operand, comptime cycles: u32) u32 {
    if (b == .A) {
        self.regs.a = 0;
        self.regs.f = .{
            .zero = true,
            .subtraction = false,
            .half_carry = false,
            .carry = false,
        };

        return cycles;
    }

    @panic("TODO");
}

// TODO
fn STOP(_: *CPU, comptime cycles: u32) u32 {
    return cycles;
}

// TODO
fn RLCA(_: *CPU, comptime cycles: u32) u32 {
    return cycles;
}

// TODO
fn RLA(_: *CPU, comptime cycles: u32) u32 {
    return cycles;
}

// TODO
fn RRCA(_: *CPU, comptime cycles: u32) u32 {
    return cycles;
}

// TODO
fn RRA(_: *CPU, comptime cycles: u32) u32 {
    return cycles;
}

// TODO
fn ADD(self: *CPU, comptime a: Operand, comptime b: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    _ = b;
    return cycles;
}

// TODO
fn SUB(self: *CPU, comptime b: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = b;
    return cycles;
}

// TODO
fn AND(self: *CPU, comptime b: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = b;
    return cycles;
}

// TODO
fn OR(self: *CPU, comptime b: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = b;
    return cycles;
}

// TODO
fn CP(self: *CPU, comptime b: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = b;
    return cycles;
}

// TODO
fn POP(self: *CPU, comptime r: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = r;
    return cycles;
}

// TODO
fn PUSH(self: *CPU, comptime r: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = r;
    return cycles;
}

// TODO
fn JR(self: *CPU, comptime cond: Condition, comptime cycles: anytype) u32 {
    switch (cond) {
        .None => {
            return cycles;
        },
        .Z => {
            return cycles.z;
        },
        .NZ => {
            if (self.regs.f.zero) {
                self.regs.pc += 1;
                return cycles.z;
            }

            const signed_value = @as(i8, @bitCast(self.mmu.read(u8, self.regs.pc)));
            const pc = @as(i16, @intCast(self.regs.pc)) +% signed_value;
            self.regs.pc = @intCast(pc);
            return cycles.nz;
        },
        .C => {
            return cycles.c;
        },
        .NC => {
            return cycles.nc;
        },
    }
}

// TODO
fn JP(self: *CPU, comptime cond: Condition, comptime a: Operand, comptime cycles: anytype) u32 {
    _ = self;
    _ = a;
    switch (cond) {
        .None => {
            return cycles;
        },
        .Z => {
            return cycles.z;
        },
        .NZ => {
            return cycles.nz;
        },
        .C => {
            return cycles.c;
        },
        .NC => {
            return cycles.nc;
        },
    }
}

// TODO
fn RET(self: *CPU, comptime cond: Condition, comptime cycles: anytype) u32 {
    _ = self;
    switch (cond) {
        .None => {
            return cycles;
        },
        .Z => {
            return cycles.z;
        },
        .NZ => {
            return cycles.nz;
        },
        .C => {
            return cycles.c;
        },
        .NC => {
            return cycles.nc;
        },
    }
}

// TODO
fn CALL(self: *CPU, comptime cond: Condition, comptime a: Operand, comptime cycles: anytype) u32 {
    _ = self;
    _ = a;
    switch (cond) {
        .None => {
            return cycles;
        },
        .Z => {
            return cycles.z;
        },
        .NZ => {
            return cycles.nz;
        },
        .C => {
            return cycles.c;
        },
        .NC => {
            return cycles.nc;
        },
    }
}

// TODO
fn DAA(self: *CPU, comptime cycles: u32) u32 {
    _ = self;
    return cycles;
}

// TODO
fn CPL(self: *CPU, comptime cycles: u32) u32 {
    _ = self;
    return cycles;
}

// TODO
fn SCF(self: *CPU, comptime cycles: u32) u32 {
    _ = self;
    return cycles;
}

// TODO
fn CCF(self: *CPU, comptime cycles: u32) u32 {
    _ = self;
    return cycles;
}

// TODO
fn HALT(self: *CPU, comptime cycles: u32) u32 {
    _ = self;
    return cycles;
}

// TODO
fn RETI(self: *CPU, comptime cycles: u32) u32 {
    _ = self;
    return cycles;
}

// TODO
fn DI(self: *CPU, comptime cycles: u32) u32 {
    _ = self;
    return cycles;
}

// TODO
fn EI(self: *CPU, comptime cycles: u32) u32 {
    _ = self;
    return cycles;
}

// TODO
fn RST(self: *CPU, comptime a: u8, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    return cycles;
}

// TODO
fn ADC(self: *CPU, comptime a: Operand, comptime b: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    _ = b;
    return cycles;
}

fn CB(self: *CPU) u32 {
    const opcode = self.fetch(u8);

    return switch (opcode) {
        0x00 => self.RLC(.B, 2),
        0x01 => self.RLC(.C, 2),
        0x02 => self.RLC(.D, 2),
        0x03 => self.RLC(.E, 2),
        0x04 => self.RLC(.H, 2),
        0x05 => self.RLC(.L, 2),
        0x06 => self.RLC(.@"(HL)", 4),
        0x07 => self.RLC(.A, 2),
        0x08 => self.RRC(.B, 2),
        0x09 => self.RRC(.C, 2),
        0x0A => self.RRC(.D, 2),
        0x0B => self.RRC(.E, 2),
        0x0C => self.RRC(.H, 2),
        0x0D => self.RRC(.L, 2),
        0x0E => self.RRC(.@"(HL)", 4),
        0x0F => self.RRC(.A, 2),
        0x10 => self.RL(.B, 2),
        0x11 => self.RL(.C, 2),
        0x12 => self.RL(.D, 2),
        0x13 => self.RL(.E, 2),
        0x14 => self.RL(.H, 2),
        0x15 => self.RL(.L, 2),
        0x16 => self.RL(.@"(HL)", 4),
        0x17 => self.RL(.A, 2),
        0x18 => self.RR(.B, 2),
        0x19 => self.RR(.C, 2),
        0x1A => self.RR(.D, 2),
        0x1B => self.RR(.E, 2),
        0x1C => self.RR(.H, 2),
        0x1D => self.RR(.L, 2),
        0x1E => self.RR(.@"(HL)", 4),
        0x1F => self.RR(.A, 2),
        0x20 => self.SLA(.B, 2),
        0x21 => self.SLA(.C, 2),
        0x22 => self.SLA(.D, 2),
        0x23 => self.SLA(.E, 2),
        0x24 => self.SLA(.H, 2),
        0x25 => self.SLA(.L, 2),
        0x26 => self.SLA(.@"(HL)", 4),
        0x27 => self.SLA(.A, 2),
        0x28 => self.SRA(.B, 2),
        0x29 => self.SRA(.C, 2),
        0x2A => self.SRA(.D, 2),
        0x2B => self.SRA(.E, 2),
        0x2C => self.SRA(.H, 2),
        0x2D => self.SRA(.L, 2),
        0x2E => self.SRA(.@"(HL)", 4),
        0x2F => self.SRA(.A, 2),
        0x30 => self.SWAP(.B, 2),
        0x31 => self.SWAP(.C, 2),
        0x32 => self.SWAP(.D, 2),
        0x33 => self.SWAP(.E, 2),
        0x34 => self.SWAP(.H, 2),
        0x35 => self.SWAP(.L, 2),
        0x36 => self.SWAP(.@"(HL)", 4),
        0x37 => self.SWAP(.A, 2),
        0x38 => self.SRL(.B, 2),
        0x39 => self.SRL(.C, 2),
        0x3A => self.SRL(.D, 2),
        0x3B => self.SRL(.E, 2),
        0x3C => self.SRL(.H, 2),
        0x3D => self.SRL(.L, 2),
        0x3E => self.SRL(.@"(HL)", 4),
        0x3F => self.SRL(.A, 2),
        0x40 => self.BIT(0, .B, 2),
        0x41 => self.BIT(0, .C, 2),
        0x42 => self.BIT(0, .D, 2),
        0x43 => self.BIT(0, .E, 2),
        0x44 => self.BIT(0, .H, 2),
        0x45 => self.BIT(0, .L, 2),
        0x46 => self.BIT(0, .@"(HL)", 3),
        0x47 => self.BIT(0, .A, 2),
        0x48 => self.BIT(1, .B, 2),
        0x49 => self.BIT(1, .C, 2),
        0x4A => self.BIT(1, .D, 2),
        0x4B => self.BIT(1, .E, 2),
        0x4C => self.BIT(1, .H, 2),
        0x4D => self.BIT(1, .L, 2),
        0x4E => self.BIT(1, .@"(HL)", 3),
        0x4F => self.BIT(1, .A, 2),
        0x50 => self.BIT(2, .B, 2),
        0x51 => self.BIT(2, .C, 2),
        0x52 => self.BIT(2, .D, 2),
        0x53 => self.BIT(2, .E, 2),
        0x54 => self.BIT(2, .H, 2),
        0x55 => self.BIT(2, .L, 2),
        0x56 => self.BIT(2, .@"(HL)", 3),
        0x57 => self.BIT(2, .A, 2),
        0x58 => self.BIT(3, .B, 2),
        0x59 => self.BIT(3, .C, 2),
        0x5A => self.BIT(3, .D, 2),
        0x5B => self.BIT(3, .E, 2),
        0x5C => self.BIT(3, .H, 2),
        0x5D => self.BIT(3, .L, 2),
        0x5E => self.BIT(3, .@"(HL)", 3),
        0x5F => self.BIT(3, .A, 2),
        0x60 => self.BIT(4, .B, 2),
        0x61 => self.BIT(4, .C, 2),
        0x62 => self.BIT(4, .D, 2),
        0x63 => self.BIT(4, .E, 2),
        0x64 => self.BIT(4, .H, 2),
        0x65 => self.BIT(4, .L, 2),
        0x66 => self.BIT(4, .@"(HL)", 3),
        0x67 => self.BIT(4, .A, 2),
        0x68 => self.BIT(5, .B, 2),
        0x69 => self.BIT(5, .C, 2),
        0x6A => self.BIT(5, .D, 2),
        0x6B => self.BIT(5, .E, 2),
        0x6C => self.BIT(5, .H, 2),
        0x6D => self.BIT(5, .L, 2),
        0x6E => self.BIT(5, .@"(HL)", 3),
        0x6F => self.BIT(5, .A, 2),
        0x70 => self.BIT(6, .B, 2),
        0x71 => self.BIT(6, .C, 2),
        0x72 => self.BIT(6, .D, 2),
        0x73 => self.BIT(6, .E, 2),
        0x74 => self.BIT(6, .H, 2),
        0x75 => self.BIT(6, .L, 2),
        0x76 => self.BIT(6, .@"(HL)", 3),
        0x77 => self.BIT(6, .A, 2),
        0x78 => self.BIT(7, .B, 2),
        0x79 => self.BIT(7, .C, 2),
        0x7A => self.BIT(7, .D, 2),
        0x7B => self.BIT(7, .E, 2),
        0x7C => self.BIT(7, .H, 2),
        0x7D => self.BIT(7, .L, 2),
        0x7E => self.BIT(7, .@"(HL)", 3),
        0x7F => self.BIT(7, .A, 2),
        0x80 => self.RES(0, .B, 2),
        0x81 => self.RES(0, .C, 2),
        0x82 => self.RES(0, .D, 2),
        0x83 => self.RES(0, .E, 2),
        0x84 => self.RES(0, .H, 2),
        0x85 => self.RES(0, .L, 2),
        0x86 => self.RES(0, .@"(HL)", 4),
        0x87 => self.RES(0, .A, 2),
        0x88 => self.RES(1, .B, 2),
        0x89 => self.RES(1, .C, 2),
        0x8A => self.RES(1, .D, 2),
        0x8B => self.RES(1, .E, 2),
        0x8C => self.RES(1, .H, 2),
        0x8D => self.RES(1, .L, 2),
        0x8E => self.RES(1, .@"(HL)", 4),
        0x8F => self.RES(1, .A, 2),
        0x90 => self.RES(2, .B, 2),
        0x91 => self.RES(2, .C, 2),
        0x92 => self.RES(2, .D, 2),
        0x93 => self.RES(2, .E, 2),
        0x94 => self.RES(2, .H, 2),
        0x95 => self.RES(2, .L, 2),
        0x96 => self.RES(2, .@"(HL)", 4),
        0x97 => self.RES(2, .A, 2),
        0x98 => self.RES(3, .B, 2),
        0x99 => self.RES(3, .C, 2),
        0x9A => self.RES(3, .D, 2),
        0x9B => self.RES(3, .E, 2),
        0x9C => self.RES(3, .H, 2),
        0x9D => self.RES(3, .L, 2),
        0x9E => self.RES(3, .@"(HL)", 4),
        0x9F => self.RES(3, .A, 2),
        0xA0 => self.RES(4, .B, 2),
        0xA1 => self.RES(4, .C, 2),
        0xA2 => self.RES(4, .D, 2),
        0xA3 => self.RES(4, .E, 2),
        0xA4 => self.RES(4, .H, 2),
        0xA5 => self.RES(4, .L, 2),
        0xA6 => self.RES(4, .@"(HL)", 4),
        0xA7 => self.RES(4, .A, 2),
        0xA8 => self.RES(5, .B, 2),
        0xA9 => self.RES(5, .C, 2),
        0xAA => self.RES(5, .D, 2),
        0xAB => self.RES(5, .E, 2),
        0xAC => self.RES(5, .H, 2),
        0xAD => self.RES(5, .L, 2),
        0xAE => self.RES(5, .@"(HL)", 4),
        0xAF => self.RES(5, .A, 2),
        0xB0 => self.RES(6, .B, 2),
        0xB1 => self.RES(6, .C, 2),
        0xB2 => self.RES(6, .D, 2),
        0xB3 => self.RES(6, .E, 2),
        0xB4 => self.RES(6, .H, 2),
        0xB5 => self.RES(6, .L, 2),
        0xB6 => self.RES(6, .@"(HL)", 4),
        0xB7 => self.RES(6, .A, 2),
        0xB8 => self.RES(7, .B, 2),
        0xB9 => self.RES(7, .C, 2),
        0xBA => self.RES(7, .D, 2),
        0xBB => self.RES(7, .E, 2),
        0xBC => self.RES(7, .H, 2),
        0xBD => self.RES(7, .L, 2),
        0xBE => self.RES(7, .@"(HL)", 4),
        0xBF => self.RES(7, .A, 2),
        0xC0 => self.SET(0, .B, 2),
        0xC1 => self.SET(0, .C, 2),
        0xC2 => self.SET(0, .D, 2),
        0xC3 => self.SET(0, .E, 2),
        0xC4 => self.SET(0, .H, 2),
        0xC5 => self.SET(0, .L, 2),
        0xC6 => self.SET(0, .@"(HL)", 4),
        0xC7 => self.SET(0, .A, 2),
        0xC8 => self.SET(1, .B, 2),
        0xC9 => self.SET(1, .C, 2),
        0xCA => self.SET(1, .D, 2),
        0xCB => self.SET(1, .E, 2),
        0xCC => self.SET(1, .H, 2),
        0xCD => self.SET(1, .L, 2),
        0xCE => self.SET(1, .@"(HL)", 4),
        0xCF => self.SET(1, .A, 2),
        0xD0 => self.SET(2, .B, 2),
        0xD1 => self.SET(2, .C, 2),
        0xD2 => self.SET(2, .D, 2),
        0xD3 => self.SET(2, .E, 2),
        0xD4 => self.SET(2, .H, 2),
        0xD5 => self.SET(2, .L, 2),
        0xD6 => self.SET(2, .@"(HL)", 4),
        0xD7 => self.SET(2, .A, 2),
        0xD8 => self.SET(3, .B, 2),
        0xD9 => self.SET(3, .C, 2),
        0xDA => self.SET(3, .D, 2),
        0xDB => self.SET(3, .E, 2),
        0xDC => self.SET(3, .H, 2),
        0xDD => self.SET(3, .L, 2),
        0xDE => self.SET(3, .@"(HL)", 4),
        0xDF => self.SET(3, .A, 2),
        0xE0 => self.SET(4, .B, 2),
        0xE1 => self.SET(4, .C, 2),
        0xE2 => self.SET(4, .D, 2),
        0xE3 => self.SET(4, .E, 2),
        0xE4 => self.SET(4, .H, 2),
        0xE5 => self.SET(4, .L, 2),
        0xE6 => self.SET(4, .@"(HL)", 4),
        0xE7 => self.SET(4, .A, 2),
        0xE8 => self.SET(5, .B, 2),
        0xE9 => self.SET(5, .C, 2),
        0xEA => self.SET(5, .D, 2),
        0xEB => self.SET(5, .E, 2),
        0xEC => self.SET(5, .H, 2),
        0xED => self.SET(5, .L, 2),
        0xEE => self.SET(5, .@"(HL)", 4),
        0xEF => self.SET(5, .A, 2),
        0xF0 => self.SET(6, .B, 2),
        0xF1 => self.SET(6, .C, 2),
        0xF2 => self.SET(6, .D, 2),
        0xF3 => self.SET(6, .E, 2),
        0xF4 => self.SET(6, .H, 2),
        0xF5 => self.SET(6, .L, 2),
        0xF6 => self.SET(6, .@"(HL)", 4),
        0xF7 => self.SET(6, .A, 2),
        0xF8 => self.SET(7, .B, 2),
        0xF9 => self.SET(7, .C, 2),
        0xFA => self.SET(7, .D, 2),
        0xFB => self.SET(7, .E, 2),
        0xFC => self.SET(7, .H, 2),
        0xFD => self.SET(7, .L, 2),
        0xFE => self.SET(7, .@"(HL)", 4),
        0xFF => self.SET(7, .A, 2),
    };
}

fn BIT(self: *CPU, comptime bit: u3, comptime a: Operand, comptime cycles: u16) u16 {
    const value = self.val(a);
    const bit_value: bool = @as(u1, @truncate(value >> bit)) == 0;

    self.regs.f = .{
        .zero = bit_value,
        .subtraction = false,
        .half_carry = true,
        .carry = self.regs.f.carry,
    };

    return cycles;
}

fn RR(self: *CPU, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    return cycles;
}

fn RL(self: *CPU, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    return cycles;
}

fn RLC(self: *CPU, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    return cycles;
}

fn RRC(self: *CPU, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    return cycles;
}

fn SLA(self: *CPU, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    return cycles;
}

fn SRA(self: *CPU, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    return cycles;
}

fn SWAP(self: *CPU, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    return cycles;
}

fn SRW(self: *CPU, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    return cycles;
}

fn SRL(self: *CPU, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    return cycles;
}

fn SET(self: *CPU, comptime v: u8, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    _ = v;
    return cycles;
}

fn RES(self: *CPU, comptime v: u8, comptime a: Operand, comptime cycles: u32) u32 {
    _ = self;
    _ = a;
    _ = v;
    return cycles;
}
