use std::rc::Rc;
use std::cell::RefCell;

use super::mmu::MemoryInterface;

pub struct Clock {
  pub m: u32,
  pub t: u32,
}

pub struct Registers {
  // 8 bit registers
  pub a: u8,
  pub b: u8,
  pub c: u8,
  pub d: u8,
  pub e: u8,
  pub h: u8,
  pub l: u8,

  // Flags
  pub f: u8,

  // 16 bit registers
  // program counter
  pub pc: u16,
  // stack pointer
  pub sp: u16,

  // clock for last instruction
  last_clock: Clock,
}

pub struct CPU {
  pub clock: Clock,
  pub registers: Registers,

  memory_interface: Rc<RefCell<MemoryInterface>>,
}

impl CPU {
  pub fn new(memory_interface: Rc<RefCell<MemoryInterface>>) -> CPU {
    let clock = Clock {
      m: 0, t: 0,
    };

    let registers = Registers {
      a: 0, b: 0, c: 0, d: 0, e: 0,
      h: 0, l: 0, f: 0,

      pc: 0, sp: 0,

      last_clock: Clock { m: 0, t: 0 },
    };

    CPU { clock, registers, memory_interface }
  }

  // Operations
  pub fn nop(&mut self) {
    // takes 1 M-Time
    self.registers.last_clock.m = 1;
    self.registers.last_clock.t = 4;
  }

  pub fn add(&mut self) {
    let result = (self.registers.a as u16) + (self.registers.e as u16);

    self.clear_flags();

    // check for 0
    if ((result as u8) | 255) == 0 { self.set_zero_flag(); }

    // check for carry
    if result > 255 { self.set_carry_flag(); }

    // this truncates to 8 bit automatically
    self.registers.a = result as u8;

    // takes 1 M-Time
    self.registers.last_clock.m = 1;
    self.registers.last_clock.t = 4;
  }

  pub fn cp(&mut self) {
    // TODO: Can this overflow if we use i8 instead of i16? I don't think so
    let result = (self.registers.a as i8) - (self.registers.b as i8);

    self.set_sub_flag();

    // check for 0
    if ((result as u8) | 255) == 0 { self.set_zero_flag(); }

    // check for underflow
    if result < 0 { self.set_carry_flag(); }

    // takes 1 M-Time
    self.registers.last_clock.m = 1;
    self.registers.last_clock.t = 4;
  }

  // Flags manipulation
  // TODO: Inline these?
  fn clear_flags(&mut self) { self.registers.f = 0; }
  fn set_zero_flag(&mut self) { self.registers.f = self.registers.f | 0x80; }
  fn set_carry_flag(&mut self) { self.registers.f = self.registers.f | 0x10; }
  fn set_sub_flag(&mut self) { self.registers.f = self.registers.f | 0x40; }
}
