use std::rc::Rc;
use std::cell::RefCell;

use super::registers::{Registers, ByteRegister, WordRegister};
use super::super::mmu::MemoryInterface;

pub struct Clock {
  pub m: u32,
  pub t: u32,
}

pub struct CPU {
  clock: Clock,
  registers: Registers,

  // clock for last instruction
  // TODO: should this be here or in the registers?
  last_clock: Clock,

  memory_interface: Rc<RefCell<MemoryInterface>>,
}

impl CPU {
  pub fn new(memory_interface: Rc<RefCell<MemoryInterface>>) -> CPU {
    let clock = Clock { m: 0, t: 0 };
    let registers = Registers::new();
    let last_clock = Clock { m: 0, t: 0 };

    CPU { clock, registers, memory_interface, last_clock }
  }

  // ------------------------------------
  // 8-bit loads
  // ------------------------------------

  // Put n into nn
  pub fn ld_nn_n(&mut self, dst: ByteRegister, n: u8) {
    self.registers[dst] = n;
    self.set_last_clock(2);
  }

  // Put value from src to dest
  pub fn ld_r1_r2(&mut self, dst: ByteRegister, src: ByteRegister) {
    self.registers[dst] = self.registers[src];
    self.set_last_clock(1);
  }
  pub fn ld_r1_hlm(&mut self, dst: ByteRegister) {
    self.registers[dst] = self.read_hl();
    self.set_last_clock(2);
  }
  pub fn ld_hlm_r1(&mut self, src: ByteRegister) {
    let byte = self.registers[src];
    self.write_hl(byte);
    self.set_last_clock(2);
  }
  pub fn ld_hlm_n(&mut self, value: u8) {
    self.write_hl(value);
    self.set_last_clock(3);
  }

  // Put value from n into A
  pub fn ld_a_r1(&mut self, src: ByteRegister) {
    self.registers[ByteRegister::A] = self.registers[src];
    self.set_last_clock(1);
  }
  pub fn ld_a_m(&mut self, src: WordRegister) {
    let pointer = self.registers.read_word(src);
    let byte = self.memory_interface.borrow_mut().read_byte(pointer);
    self.registers[ByteRegister::A] = byte;
    self.set_last_clock(2);
  }
  pub fn ld_a_nn(&mut self, src: u16) {
    let byte = self.memory_interface.borrow_mut().read_byte(src);
    self.registers[ByteRegister::A] = byte;
    self.set_last_clock(4);
  }

  // Put value from A into n
  pub fn ld_r1_a(&mut self, dst: ByteRegister) {
    self.registers[dst] = self.registers[ByteRegister::A];
    self.set_last_clock(1);
  }
  pub fn ld_m_a(&mut self, dst: WordRegister) {
    let pointer = self.registers.read_word(dst);
    let byte = self.registers[ByteRegister::A];
    self.memory_interface.borrow_mut().write_byte(pointer, byte);
    self.set_last_clock(2);
  }
  pub fn ld_nn_a(&mut self, dst: u16) {
    let byte = self.registers[ByteRegister::A];
    self.memory_interface.borrow_mut().write_byte(dst, byte);
    self.set_last_clock(4);
  }

  // Put value at address 0xFF00 + C into A
  pub fn ld_a_cm(&mut self) {
    let pointer = 0xFF00 + (self.registers[ByteRegister::C] as u16);
    self.registers[ByteRegister::A] = self.memory_interface.borrow().read_byte(pointer);
    self.set_last_clock(2);
  }

  // Put value at A into 0xFF00 + C
  pub fn ld_cm_a(&mut self) {
    let byte = self.registers[ByteRegister::A];
    let pointer = 0xFF00 + (self.registers[ByteRegister::C] as u16);
    self.memory_interface.borrow_mut().write_byte(pointer, byte);
    self.set_last_clock(2);
  }

  // Put value from HL into A and decrement HL
  pub fn ldd_a_hlm(&mut self) {
    let pointer = self.registers.read_word(WordRegister::HL);
    self.registers.write_word(WordRegister::HL, pointer - 1);
    self.registers[ByteRegister::A] = self.memory_interface.borrow().read_byte(pointer);
    self.set_last_clock(2);
  }

  // Put value from A into HL and decrement HL
  pub fn ldd_hlm_a(&mut self) {
    let pointer = self.registers.read_word(WordRegister::HL);
    let byte = self.registers[ByteRegister::A];
    self.write_hl(byte);

    self.registers.write_word(WordRegister::HL, pointer - 1);
    self.set_last_clock(2);
  }

  // Put value from HL into A and increment HL
  pub fn ldi_a_hlm(&mut self) {
    let pointer = self.registers.read_word(WordRegister::HL);
    self.registers.write_word(WordRegister::HL, pointer + 1);
    self.registers[ByteRegister::A] = self.memory_interface.borrow().read_byte(pointer);
    self.set_last_clock(2);
  }

  // Put value from A into HL and increment HL
  pub fn ldi_hlm_a(&mut self) {
    let pointer = self.registers.read_word(WordRegister::HL);
    let byte = self.registers[ByteRegister::A];
    self.write_hl(byte);

    self.registers.write_word(WordRegister::HL, pointer + 1);
    self.set_last_clock(2);
  }

  // Put value at address 0xFF00 + n into A
  pub fn ldh_a_nm(&mut self, n: u8) {
    let pointer = 0xFF00 + (n as u16);
    self.registers[ByteRegister::A] = self.memory_interface.borrow().read_byte(pointer);
    self.set_last_clock(3);
  }

  // Put value at A into 0xFF00 + n
  pub fn ldh_nm_a(&mut self, n: u8) {
    let byte = self.registers[ByteRegister::A];
    let pointer = 0xFF00 + (n as u16);
    self.memory_interface.borrow_mut().write_byte(pointer, byte);
    self.set_last_clock(3);
  }

  // ------------------------------------
  // 16-bit loads
  // ------------------------------------

  // Put value nn into n
  pub fn ld_n_nn(&mut self, dst: WordRegister, n: u16) {
    self.registers.write_word(dst, n);
    self.set_last_clock(3);
  }

  // Put hl into stack pointer
  pub fn ld_sp_hl(&mut self) {
    let word = self.registers.read_word(WordRegister::HL);
    self.registers.write_word(WordRegister::SP, word);
    self.set_last_clock(2);
  }

  // TODO: LD HL,SP+m

  // Put stack pointer into address nn
  pub fn ld_nnm_sp(&mut self, dst: u16) {
    let word = self.registers.read_word(WordRegister::SP);
    self.memory_interface.borrow_mut().write_word(dst, word);
    self.set_last_clock(5);
  }

  // Push register pair nn onto stack, decrease SP twice
  pub fn push_nn(&mut self, src: WordRegister) {
    let sp = self.registers.read_word(WordRegister::SP);
    let word = self.registers.read_word(src);
    self.memory_interface.borrow_mut().write_word(sp, word);
    self.registers.write_word(WordRegister::SP, sp - 2);
    self.set_last_clock(4);
  }

  // Pop word off stack into register pair nn, increment SP twice
  pub fn pop_nn(&mut self, dst: WordRegister) {
    let sp = self.registers.read_word(WordRegister::SP);
    let word = self.memory_interface.borrow().read_word(sp);
    self.registers.write_word(dst, word);
    self.registers.write_word(WordRegister::SP, sp + 2);
    self.set_last_clock(3);
  }

  // ------------------------------------
  // Others
  // ------------------------------------

  pub fn nop(&mut self) {
    self.set_last_clock(1);
  }

  // takes M-Time as input
  fn set_last_clock(&mut self, m_time: u32) {
    self.last_clock.m = m_time;
    self.last_clock.t = m_time * 4;
  }

  // helpers
  // TODO: Check which of these should be inlined for performance
  fn read_hl(&self) -> u8 {
    let pointer = self.registers.read_word(WordRegister::HL);
    self.memory_interface.borrow().read_byte(pointer)
  }

  fn write_hl(&mut self, byte: u8) {
    let pointer = self.registers.read_word(WordRegister::HL);
    self.memory_interface.borrow_mut().write_byte(pointer, byte);
  }

  fn read_pc(&self) -> u8 {
    let pointer = self.registers.read_word(WordRegister::PC);
    self.memory_interface.borrow().read_byte(pointer)
  }

  fn set_add_flags(&mut self, result: u16) {
    self.registers.clear_flags();

    // check for 0
    if ((result as u8) | 255) == 0 { self.registers.set_zero_flag(); }

    // check for carry
    if result > 255 { self.registers.set_carry_flag(); }
  }
}
