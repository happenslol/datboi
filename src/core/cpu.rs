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
  // 8-bit ALU
  // ------------------------------------

  // Add n to A
  pub fn add_n(&mut self, src: ByteRegister) {
    let result = (self.registers[ByteRegister::A] as u16) +
      (self.registers[src] as u16);

    self.registers.clear_flags();
    if result == 0 { self.registers.set_zero_flag(); }
    if result > 255 { self.registers.set_carry_flag(); }
    if (check_half_carry_add8(self.registers[ByteRegister::A], self.registers[src])) {
      self.registers.set_half_carry_flag();
    }

    self.registers[ByteRegister::A] = result as u8;
    self.set_last_clock(1);
  }
  pub fn add_hlm(&mut self) {
    let result = (self.registers[ByteRegister::A] as u16) +
      (self.read_hl() as u16);

    self.registers.clear_flags();
    if result == 0 { self.registers.set_zero_flag(); }
    if result > 255 { self.registers.set_carry_flag(); }
    if (check_half_carry_add8(self.registers[ByteRegister::A], self.read_hl())) {
      self.registers.set_half_carry_flag();
    }

    self.registers[ByteRegister::A] = result as u8;
    self.set_last_clock(2);
  }

  // Sub n from A
  pub fn sub_n(&mut self, src: ByteRegister) {
    let result = (self.registers[ByteRegister::A] as i16) -
      (self.registers[src] as i16);
    
    self.registers.clear_flags();
    self.registers.set_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    if result < 0 { self.registers.set_carry_flag(); }
    if (check_half_carry_sub8(self.registers[ByteRegister::A], self.registers[src])) {
      self.registers.set_half_carry_flag();
    }

    self.registers[ByteRegister::A] = result as u8;
    self.set_last_clock(1);
  }
  pub fn sub_hlm(&mut self) {
    let result = (self.registers[ByteRegister::A] as i16) -
      (self.read_hl() as i16);
    
    self.registers.clear_flags();
    self.registers.set_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    if result < 0 { self.registers.set_carry_flag(); }
    if (check_half_carry_sub8(self.registers[ByteRegister::A], self.read_hl())) {
      self.registers.set_half_carry_flag();
    }

    self.registers[ByteRegister::A] = result as u8;
    self.set_last_clock(2);
  }

  // TODO: SBC A,n

  // AND n with a
  pub fn and_n(&mut self, src: ByteRegister) {
    let result = self.registers[ByteRegister::A] & self.registers[src];

    self.registers.clear_flags();
    if result == 0 { self.registers.set_zero_flag(); }
    self.registers.set_half_carry_flag();

    self.registers[ByteRegister::A] = result;
    self.set_last_clock(1);
  }
  pub fn and_hlm(&mut self) {
    let result = self.registers[ByteRegister::A] & self.read_hl();

    self.registers.clear_flags();
    if result == 0 { self.registers.set_zero_flag(); }
    self.registers.set_half_carry_flag();

    self.registers[ByteRegister::A] = result;
    self.set_last_clock(2);
  }

  // OR n with a
  pub fn or_n(&mut self, src: ByteRegister) {
    let result = self.registers[ByteRegister::A] | self.registers[src];

    self.registers.clear_flags();
    if result == 0 { self.registers.set_zero_flag(); }

    self.registers[ByteRegister::A] = result;
    self.set_last_clock(1);
  }
  pub fn or_hlm(&mut self) {
    let result = self.registers[ByteRegister::A] | self.read_hl();

    self.registers.clear_flags();
    if result == 0 { self.registers.set_zero_flag(); }

    self.registers[ByteRegister::A] = result;
    self.set_last_clock(2);
  }

  // XOR n with a
  pub fn xor_n(&mut self, src: ByteRegister) {
    let result = self.registers[ByteRegister::A] ^ self.registers[src];

    self.registers.clear_flags();
    if result == 0 { self.registers.set_zero_flag(); }

    self.registers[ByteRegister::A] = result;
    self.set_last_clock(1);
  }
  pub fn xor_hlm(&mut self) {
    let result = self.registers[ByteRegister::A] ^ self.read_hl();

    self.registers.clear_flags();
    if result == 0 { self.registers.set_zero_flag(); }

    self.registers[ByteRegister::A] = result;
    self.set_last_clock(2);
  }

  // cp n with A
  pub fn cp_r1(&mut self, src: ByteRegister) {
    let result = (self.registers[ByteRegister::A] as i16) - (self.registers[src] as i16);
    
    self.registers.clear_flags();
    self.registers.set_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    if result < 0 { self.registers.set_carry_flag(); }
    if (check_half_carry_sub8(self.registers[ByteRegister::A], self.registers[src])) {
      self.registers.set_half_carry_flag();
    }

    self.set_last_clock(1);
  }
  pub fn cp_hlm(&mut self) {
    let result = (self.registers[ByteRegister::A] as i16) - (self.read_hl() as i16);
    
    self.registers.clear_flags();
    self.registers.set_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    if result < 0 { self.registers.set_carry_flag(); }
    if (check_half_carry_sub8(self.registers[ByteRegister::A], self.read_hl())) {
      self.registers.set_half_carry_flag();
    }

    self.set_last_clock(2);
  }

  // increase n by 1
  pub fn inc_n(&mut self, dst: ByteRegister) {
    let result = (self.registers[dst] as u16) + 1;

    self.registers.unset_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }
    if (check_half_carry_add8(self.registers[dst], 1)) {
      self.registers.set_half_carry_flag();
    } else {
      self.registers.unset_half_carry_flag();
    }

    self.registers[dst] = result as u8;
    self.set_last_clock(1);
  }
  pub fn inc_hlm(&mut self) {
    let result = (self.read_hl() as u16) + 1;

    self.registers.unset_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }
    if (check_half_carry_add8(self.read_hl(), 1)) {
      self.registers.set_half_carry_flag();
    } else {
      self.registers.unset_half_carry_flag();
    }

    self.write_hl(result as u8);
    self.set_last_clock(3);
  }

  // decrease n by 1
  pub fn dec_n(&mut self, dst: ByteRegister) {
    let result = (self.registers[dst] as u16) - 1;

    self.registers.unset_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }
    if (check_half_carry_sub8(self.registers[dst], 1)) {
      self.registers.set_half_carry_flag();
    } else {
      self.registers.unset_half_carry_flag();
    }

    self.registers[dst] = result as u8;
    self.set_last_clock(1);
  }
  pub fn dec_hlm(&mut self) {
    let result = (self.read_hl() as u16) - 1;

    self.registers.unset_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }
    if (check_half_carry_sub8(self.read_hl(), 1)) {
      self.registers.set_half_carry_flag();
    } else {
      self.registers.unset_half_carry_flag();
    }

    self.write_hl(result as u8);
    self.set_last_clock(3);
  }

  // add n to HL
  pub fn add_hlm_n(&mut self, src: WordRegister) {
    let a = self.registers.read_word(WordRegister::HL);
    let b = self.registers.read_word(src);
    let result = (a as u32) + (b as u32);

    self.registers.unset_sub_flag();
    if check_carry_add16(a, b) { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
    if check_half_carry_add16(a, b) {self.registers.set_half_carry_flag(); }
    else { self.registers.unset_half_carry_flag(); }

    self.registers.write_word(WordRegister::HL, result as u16);
    self.set_last_clock(2);
  }

  // TODO: ADD SP,n

  // increase nn
  pub fn inc_nn(&mut self, dst: WordRegister) {
    let result = self.registers.read_word(dst) + 1;
    self.registers.write_word(dst, result);
    self.set_last_clock(2);
  }

  // decrease nn
  pub fn dec_nn(&mut self, dst: WordRegister) {
    let result = self.registers.read_word(dst) - 1;
    self.registers.write_word(dst, result);
    self.set_last_clock(2);
  }

  // ------------------------------------
  // Miscellaneous
  // ------------------------------------

  // swap upper and lower nibles of n
  pub fn swap_n(&mut self, dst: ByteRegister) {
    let result = (self.registers[dst] << 2) | (self.registers[dst] >> 2);
    self.registers.clear_flags();
    if result == 0 { self.registers.set_zero_flag(); }
    self.registers[dst] = result;
    self.set_last_clock(2);
  }
  pub fn swap_hlm(&mut self) {
    let value = self.read_hl();
    let result = (value << 2) | (value >> 2);
    self.registers.clear_flags();
    if result == 0 { self.registers.set_zero_flag(); }
    self.write_hl(result);
    self.set_last_clock(4);
  }

  // TODO: DAA

  // complement a register (flip all bits)
  pub fn cpl(&mut self) {
    let result = !self.registers[ByteRegister::A];
    self.registers.set_sub_flag();
    self.registers.set_half_carry_flag();
    self.registers[ByteRegister::A] = result;
    self.set_last_clock(1);
  }

  // complement carry flag
  pub fn ccf(&mut self) {
    self.registers.complement_carry_flag();
    self.registers.unset_sub_flag();
    self.registers.unset_half_carry_flag();
    self.set_last_clock(1);
  }

  // set carry flag
  pub fn scf(&mut self) {
    self.registers.set_carry_flag();
    self.registers.unset_sub_flag();
    self.registers.unset_half_carry_flag();
    self.set_last_clock(1);
  }

  // no op
  pub fn nop(&mut self) {
    self.set_last_clock(1);
  }

  // stop cpu until interrupt
  pub fn halt(&mut self) {
    self.set_last_clock(1);
  }

  // stop cpu/screen until button pressed
  pub fn stop(&mut self) {
    self.set_last_clock(1);
  }

  // TODO: Disable/enable interrupts after next instruction
  pub fn di(&mut self) {}
  pub fn ei(&mut self) {}

  // ------------------------------------
  // Utility functions
  // ------------------------------------

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
}

fn check_half_carry_add8(a: u8, b: u8) -> bool {
  (((a & 0xF) + (b & 0xF)) & 0x10) == 0x10
}

fn check_half_carry_sub8(a: u8, b: u8) -> bool {
  ((((a as i16) & 0xF) - ((b as i16) & 0xF)) < 0)  
}

// TODO
fn check_carry_add16(a: u16, b: u16) -> bool { true }

// TODO
fn check_half_carry_add16(a: u16, b: u16) -> bool { true }
