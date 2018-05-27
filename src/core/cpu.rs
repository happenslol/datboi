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
  // NEW INSTRUCTIONS
  // ------------------------------------

  // ------------------------------------
  // 8-bit loads
  // ------------------------------------

  // Put nn into n
  pub fn ld_nn_n(&mut self) {
  }

  // ------------------------------------
  // Data processing instructions
  // ------------------------------------

  // adds register src to A and saves the result in A
  pub fn add(&mut self, src: ByteRegister) {
    let result = (self.registers[ByteRegister::A] as u16) +
                 (self.registers[src] as u16);

    self.set_add_flags(result);

    // this truncates to 8 bit automatically
    self.registers[ByteRegister::A] = result as u8;

    self.set_last_clock(1);
  }

  // add byte at memory location HL to A
  pub fn add_hl(&mut self) {
    let result = (self.registers[ByteRegister::A] as u16) +
                 (self.read_hl() as u16);

    self.set_add_flags(result);
    self.registers[ByteRegister::A] = result as u8;

    self.set_last_clock(1);
  }

  // add byte at memory location PC to A
  pub fn add_n(&mut self) {
    let result = (self.registers[ByteRegister::A] as u16) +
                 (self.read_pc() as u16);

    self.registers.advance_pc(1);
    self.set_add_flags(result);
    self.registers[ByteRegister::A] = result as u8;

    self.set_last_clock(2);
  }

  // adds word from word register to hl
  pub fn add_hl_w(&mut self, src: WordRegister) {
    let result = (self.registers.read_word(WordRegister::HL) as u32) +
                 (self.registers.read_word(src) as u32);

    if result > 65535 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }

    self.registers.write_word(WordRegister::HL, result as u16);
    self.set_last_clock(3);
  }

  // add byte found at 
  pub fn add_sp_n(&mut self) {

  }

  pub fn cp(&mut self) {
    // TODO: Can this overflow if we use i8 instead of i16? I don't think so
    let result = (self.registers[ByteRegister::A] as i8) -
                 (self.registers[ByteRegister::B] as i8);

    self.registers.set_sub_flag();

    // check for 0
    if ((result as u8) | 255) == 0 { self.registers.set_zero_flag(); }

    // check for underflow
    if result < 0 { self.registers.set_carry_flag(); }

    self.set_last_clock(1);
  }

  // ------------------------------------
  // Memory handling instructions
  // ------------------------------------

  // copy byte from register to register
  pub fn ld_rr(&mut self, src: ByteRegister, dst: ByteRegister) {
    self.registers[src] = self.registers[dst];
    self.set_last_clock(1);
  }

  // copy byte from location HL to register
  pub fn ld_r_hlm(&mut self, dst: ByteRegister) {
    let pointer = self.registers.read_word(WordRegister::HL);
    self.registers[dst] = self.memory_interface.borrow().read_byte(pointer);
    self.set_last_clock(2);
  }

  // write byte from register to location HL
  pub fn ld_hlm_r(&mut self, src: ByteRegister) {
    let pointer = self.registers.read_word(WordRegister::HL);
    self.memory_interface.borrow_mut().write_byte(pointer, self.registers[src]);
    self.set_last_clock(2);
  }

  // read byte from program to register
  pub fn ld_r_n(&mut self, dst: ByteRegister) {
    self.registers[dst] = self.memory_interface
      .borrow().read_byte(self.registers.read_word(WordRegister::PC));

    self.registers.advance_pc(1);
    self.set_last_clock(2);
  }

  // write byte from program to location HL
  pub fn ld_hlm_n(&mut self) {
    let pointer = self.registers.read_word(WordRegister::HL);

    let byte = self.memory_interface.borrow()
      .read_byte(self.registers.read_word(WordRegister::PC));

    self.memory_interface.borrow_mut().write_byte(pointer, byte);
    self.registers.advance_pc(1);
    self.set_last_clock(3);
  }

  // write byte to location BC from register a
  pub fn ld_bcm_a(&mut self) {
    let pointer = self.registers.read_word(WordRegister::BC);
    let byte = self.registers[ByteRegister::A];

    self.memory_interface.borrow_mut().write_byte(pointer, byte);
    self.set_last_clock(2);
  }

  // write byte to location DE from register a
  pub fn ld_dem_a(&mut self) {
    let pointer = self.registers.read_word(WordRegister::DE);
    let byte = self.registers[ByteRegister::A];

    self.memory_interface.borrow_mut().write_byte(pointer, byte);
    self.set_last_clock(2);
  }

  // write byte to location found at current program counter
  // from register a (advances pc by 2)
  pub fn ld_mm_a(&mut self) {
    let pc = self.registers.read_word(WordRegister::PC);
    let pointer = self.memory_interface.borrow().read_word(pc);
    let byte = self.registers[ByteRegister::A];

    self.memory_interface.borrow_mut().write_byte(pointer, byte);

    // TODO: Make this nicer
    self.registers.advance_pc(2);

    self.set_last_clock(4);
  }

  // load byte to register a from location BC
  pub fn ld_a_bcm(&mut self) {
    let pointer = self.registers.read_word(WordRegister::BC);
    self.registers[ByteRegister::A] = self.memory_interface.borrow().read_byte(pointer);
    self.set_last_clock(2);
  }

  // load byte to register a from location DE
  pub fn ld_a_dem(&mut self) {
    let pointer = self.registers.read_word(WordRegister::DE);
    self.registers[ByteRegister::A] = self.memory_interface.borrow().read_byte(pointer);
    self.set_last_clock(2);
  }

  // load byte from location found at current program counter
  // into register a (advances pc by 2)
  pub fn ld_a_mm(&mut self) {
    let pc = self.registers.read_word(WordRegister::PC);
    let pointer = self.memory_interface.borrow().read_word(pc);
    self.registers[ByteRegister::A] = self.memory_interface.borrow().read_byte(pointer);

    self.registers.advance_pc(2);

    self.set_last_clock(4);
  }

  // load word from memory into word registers (except PC)
  pub fn ld_r_nn(&mut self, reg: WordRegister) {
    // TODO: Does this actually need to be caught?
    if reg == WordRegister::PC { panic!("can't ld_r_nn into PC location"); }

    // TODO: Does this happen in the correct order?
    let pc = self.registers.read_word(WordRegister::PC);
    let word = self.memory_interface.borrow().read_word(pc);
    self.registers.write_word(reg, word);

    self.registers.advance_pc(2);

    self.set_last_clock(3);
  }

  // load word from location found at current program counter
  // into register HL
  pub fn ld_hlm_m(&mut self) {
    let pc = self.registers.read_word(WordRegister::PC);
    self.registers.advance_pc(2);

    let pointer = self.memory_interface.borrow().read_word(pc);
    let word = self.memory_interface.borrow().read_word(pointer);
    self.registers.write_word(WordRegister::HL, word);

    self.set_last_clock(5);
  }

  // write word to location found at current program counter
  // from register HL
  pub fn ld_m_hlm(&mut self) {
    let pc = self.registers.read_word(WordRegister::PC);
    self.registers.advance_pc(2);

    let pointer = self.memory_interface.borrow().read_word(pc);
    let word = self.registers.read_word(WordRegister::HL);

    self.memory_interface.borrow_mut().write_word(pointer, word);
    self.set_last_clock(5);
  }

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
