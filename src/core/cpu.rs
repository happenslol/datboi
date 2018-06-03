use std::rc::Rc;
use std::cell::RefCell;

use super::registers::{Registers, ByteRegister, WordRegister, Flag};
use super::super::mmu::MemoryInterface;

pub struct Clock {
  pub m: u32,
  pub t: u32,
}

pub struct CPU {
  clock: Clock,
  registers: Registers,

  interrupts_enabled: bool,
  interrupts_enabled_after_next: bool,
  in_standby: bool,
  should_power_down: bool,

  // clock for last instruction
  // TODO: should this be here or in the registers?
  last_clock: Clock,

  // Hardware interfaces
  memory_interface: Rc<RefCell<MemoryInterface>>,
}

impl CPU {
  pub fn new(memory_interface: Rc<RefCell<MemoryInterface>>) -> CPU {
    let clock = Clock { m: 0, t: 0 };
    let registers = Registers::new();
    let last_clock = Clock { m: 0, t: 0 };

    CPU {
      clock,
      registers,
      interrupts_enabled: true,
      interrupts_enabled_after_next: false,
      in_standby: false,
      should_power_down: false,
      memory_interface,
      last_clock,
    }
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
    self.registers.write_word(WordRegister::HL, pointer.wrapping_sub(1));
    self.registers[ByteRegister::A] = self.memory_interface.borrow().read_byte(pointer);
    self.set_last_clock(2);
  }

  // Put value from A into HL and decrement HL
  pub fn ldd_hlm_a(&mut self) {
    let pointer = self.registers.read_word(WordRegister::HL);
    let byte = self.registers[ByteRegister::A];
    self.write_hl(byte);

    self.registers.write_word(WordRegister::HL, pointer.wrapping_sub(1));
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

  // load sp + n into hl
  pub fn ld_hl_sp_n(&mut self) {
    let sp = self.registers.read_word(WordRegister::SP);
    let next_byte = self.next_byte() as u16;

    let operands = (sp, next_byte);

    let result = operands.0.wrapping_add(operands.1);

    self.registers.clear_flags();
    if check_carry_add16(operands) { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }

    if check_half_carry_add16(operands) { self.registers.set_half_carry_flag(); }
    else { self.registers.unset_half_carry_flag(); }

    self.registers.write_word(WordRegister::HL, result);
    self.set_last_clock(3);
  }

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
    self.registers.write_word(WordRegister::SP, sp.wrapping_sub(2));
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
    let operands = (self.registers[ByteRegister::A], self.registers[src]);
    let result = operands.0.wrapping_add(operands.1);

    self.registers.clear_flags();
    self.set_flags_add8(operands);

    self.registers[ByteRegister::A] = result as u8;
    self.set_last_clock(1);
  }
  pub fn add_hlm(&mut self) {
    let operands = (self.registers[ByteRegister::A], self.read_hl());

    self.registers.clear_flags();
    self.set_flags_add8(operands);

    self.registers[ByteRegister::A] = operands.0.wrapping_add(operands.1);
    self.set_last_clock(2);
  }

  // Sub n from A
  pub fn sub_n(&mut self, src: ByteRegister) {
    let operands = (self.registers[ByteRegister::A], self.registers[src]);
    
    self.registers.clear_flags();
    self.set_flags_sub8(operands);

    self.registers[ByteRegister::A] = operands.0.wrapping_sub(operands.1);
    self.set_last_clock(1);
  }
  pub fn sub_hlm(&mut self) {
    let operands = (self.registers[ByteRegister::A], self.read_hl());

    self.registers.clear_flags();
    self.set_flags_sub8(operands);

    self.registers[ByteRegister::A] = operands.0.wrapping_sub(operands.1);
    self.set_last_clock(2);
  }

  // subtract n and carry flag from A
  pub fn sbc_a_n(&mut self, n: ByteRegister) {
    self.set_last_clock(1);
  }
  pub fn sbc_a_hlm(&mut self) {
    self.set_last_clock(2);
  }

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
    let operands = (self.registers[ByteRegister::A], self.registers[src]);
    
    self.registers.clear_flags();
    self.set_flags_sub8(operands);

    self.set_last_clock(1);
  }
  pub fn cp_hlm(&mut self) {
    let operands = (self.registers[ByteRegister::A], self.read_hl());
    
    self.registers.clear_flags();
    self.set_flags_sub8(operands);

    self.set_last_clock(2);
  }

  // increase n by 1
  pub fn inc_n(&mut self, dst: ByteRegister) {
    let operands = (self.registers[dst], 1);
    let result = operands.0.wrapping_add(operands.1);

    self.registers.unset_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }
    if (check_half_carry_add8(operands)) {
      self.registers.set_half_carry_flag();
    } else {
      self.registers.unset_half_carry_flag();
    }

    self.registers[dst] = result;
    self.set_last_clock(1);
  }
  pub fn inc_hlm(&mut self) {
    let operands = (self.read_hl(), 1);
    let result = operands.0.wrapping_add(operands.1);

    self.registers.unset_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }
    if (check_half_carry_add8(operands)) {
      self.registers.set_half_carry_flag();
    } else {
      self.registers.unset_half_carry_flag();
    }

    self.write_hl(result);
    self.set_last_clock(3);
  }

  // decrease n by 1
  pub fn dec_n(&mut self, dst: ByteRegister) {
    let operands = (self.registers[dst], 1);
    let result = operands.0.wrapping_sub(operands.1);

    self.registers.unset_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }
    if (check_half_carry_sub8(operands)) {
      self.registers.set_half_carry_flag();
    } else {
      self.registers.unset_half_carry_flag();
    }

    self.registers[dst] = result as u8;
    self.set_last_clock(1);
  }
  pub fn dec_hlm(&mut self) {
    let operands = (self.read_hl(), 1);
    let result = operands.0.wrapping_sub(operands.1);

    self.registers.unset_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }
    if (check_half_carry_sub8(operands)) {
      self.registers.set_half_carry_flag();
    } else {
      self.registers.unset_half_carry_flag();
    }

    self.write_hl(result);
    self.set_last_clock(3);
  }

  // add n to HL
  pub fn add_hlm_n(&mut self, src: WordRegister) {
    let operands = (self.registers.read_word(WordRegister::HL),
                    self.registers.read_word(src));
    let result = (operands.0 as u32) + (operands.1 as u32);

    self.registers.unset_sub_flag();
    if check_carry_add16(operands) { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
    if check_half_carry_add16(operands) {self.registers.set_half_carry_flag(); }
    else { self.registers.unset_half_carry_flag(); }

    self.registers.write_word(WordRegister::HL, result as u16);
    self.set_last_clock(2);
  }

  // add n to stack pointer
  pub fn add_sp_n(&mut self) {
    let sp = self.registers.read_word(WordRegister::SP);
    let next_byte = self.next_byte() as u16;

    let operands = (sp, next_byte);
    let result = operands.0.wrapping_add(operands.1);

    self.registers.clear_flags();
    if check_carry_add16(operands) { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
    if check_half_carry_add16(operands) {self.registers.set_half_carry_flag(); }
    else { self.registers.unset_half_carry_flag(); }

    self.registers.write_word(WordRegister::SP, result);
    self.set_last_clock(4);
  }

  // increase nn
  pub fn inc_nn(&mut self, dst: WordRegister) {
    let result = self.registers.read_word(dst) + 1;
    self.registers.write_word(dst, result);
    self.set_last_clock(2);
  }

  // decrease nn
  pub fn dec_nn(&mut self, dst: WordRegister) {
    let result = self.registers.read_word(dst).wrapping_sub(1);
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

  // decimal adjust register a
  pub fn daa(&mut self) {
    let a = self.registers[ByteRegister::A];

    let mut adjust = 0;

    // check if we need to adjust for halfcarry
    if self.registers.get_flag(Flag::HalfCarry) { adjust |= 0x06; }

    // check if we need to adjust for carry
    if self.registers.get_flag(Flag::Carry) { adjust |= 0x60; }

    let result = if self.registers.get_flag(Flag::Sub) {
      a.wrapping_sub(adjust)
    } else {
      // TODO: Understand this at some point
      if a & 0x0F > 0x09 { adjust |= 0x06; }
      if a > 0x99 { adjust |= 0x60; }

      a.wrapping_sub(adjust)
    };

    self.registers[ByteRegister::A] = result;
    self.registers.unset_half_carry_flag();

    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }

    if (adjust & 0x60 != 0) { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }

    self.set_last_clock(1);
  }

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

  // disable interrupts after next instruction
  pub fn di(&mut self) {
    self.interrupts_enabled = false;
    self.set_last_clock(1);
  }

  // enable interrupts after next instruction
  pub fn ei(&mut self) {
    self.interrupts_enabled_after_next = true;
    self.set_last_clock(1);
  }

  // ------------------------------------
  // Rotates/Shifts
  // ------------------------------------

  // rotate A left
  pub fn rlca(&mut self) {
    let a = self.registers[ByteRegister::A];

    let msb = a >> 7;
    let result = (a << 1) | msb;

    self.registers[ByteRegister::A] = result;

    self.registers.clear_flags();
    if msb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
  }

  // rotate A left through carry
  pub fn rla(&mut self) {
    let a = self.registers[ByteRegister::A];

    let new_carry = (a >> 7) != 0;
    // TODO: Check if this is correct
    let old_carry = (self.registers.f & 0x10) >> 4;

    self.registers[ByteRegister::A] = (a << 1) | old_carry;

    self.registers.clear_flags();
    if new_carry { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
  }

  // rotate A right
  pub fn rrca(&mut self) {
    let a = self.registers[ByteRegister::A];

    let lsb = a & 1;

    self.registers[ByteRegister::A] = (a >> 1) | (lsb << 7);

    self.registers.clear_flags();
    if lsb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
  }

  // rotate A right through carry
  pub fn rra(&mut self) {
    let a = self.registers[ByteRegister::A];

    let new_carry = (a & 1) != 0;
    // TODO: Check if this is correct
    let old_carry = (self.registers.f & 0x10) >> 4;

    self.registers[ByteRegister::A] = (a >> 1) | (old_carry << 7);

    self.registers.clear_flags();
    if new_carry { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
  }

  // rotate n left
  pub fn rlc_n(&mut self, n: ByteRegister) {
    let a = self.registers[n];

    let msb = a >> 7;
    let result = (a << 1) | msb;

    self.registers[n] = result;

    self.registers.clear_flags();
    if msb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
  }

  // rotate n left through carry
  pub fn rl_n(&mut self, n: ByteRegister) {
    let a = self.registers[n];

    let new_carry = (a >> 7) != 0;
    // TODO: Check if this is correct
    let old_carry = (self.registers.f & 0x10) >> 4;

    self.registers[n] = (a << 1) | old_carry;

    self.registers.clear_flags();
    if new_carry { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
  }

  // rotate n right
  pub fn rrc_n(&mut self, n: ByteRegister) {
    let a = self.registers[n];

    let lsb = a & 1;

    self.registers[n] = (a >> 1) | (lsb << 7);

    self.registers.clear_flags();
    if lsb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
  }

  // rotate n right through carry
  pub fn rr_n(&mut self, n: ByteRegister) {
    let a = self.registers[n];

    let new_carry = (a & 1) != 0;
    // TODO: Check if this is correct
    let old_carry = self.registers.get_flag(Flag::Carry) as u8;

    self.registers[n] = (a >> 1) | (old_carry << 7);

    self.registers.clear_flags();
    if new_carry { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
  }

  // shift n left into carry, lsb of n to 0
  pub fn sla_n(&mut self, n: ByteRegister) {
    let value = self.registers[n];
    let msb = value >> 7;

    if msb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }

    self.registers[n] = (value << 1) & 0xFE;
    self.set_last_clock(2);
  }
  pub fn sla_hlm(&mut self) {
    let value = self.read_hl();
    let msb = value >> 7;

    if msb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }

    self.write_hl((value << 1) & 0xFE);
    self.set_last_clock(4);
  }

  // shift n right into carry, keep msb of n
  pub fn sra_n(&mut self, n: ByteRegister) {
    let value = self.registers[n];
    let msb = value >> 7;
    let lsb = value & 1;

    if lsb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }

    self.registers[n] = (value << 1) | (msb << 7);
    self.set_last_clock(2);
  }
  pub fn sra_hlm(&mut self) {
    let value = self.read_hl();
    let msb = value >> 7;
    let lsb = value & 1;

    if lsb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }

    self.write_hl((value << 1) | (msb << 7));
    self.set_last_clock(4);
  }

  // shift n right into carry, msb of n to 0
  pub fn srl_n(&mut self, n: ByteRegister) {
    let value = self.registers[n];
    let lsb = value & 1;

    if lsb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }

    self.registers[n] = value << 1;
    self.set_last_clock(2);
  }
  pub fn srl_hlm(&mut self) {
    let value = self.read_hl();
    let lsb = value & 1;

    if lsb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }

    self.write_hl(value << 1);
    self.set_last_clock(4);
  }

  // ------------------------------------
  // Bit Operations
  // ------------------------------------

  // test bit b in register n
  pub fn bit_b_n(&mut self, n: ByteRegister, b: u8) {
    let result = (self.registers[n] >> b) & 1u8;
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }

    self.registers.unset_sub_flag();
    self.registers.set_half_carry_flag();

    self.set_last_clock(2);
  }
  pub fn bit_b_hlm(&mut self, b: u8) {
    let result = (self.read_hl() >> b) & 1u8;
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }

    self.registers.unset_sub_flag();
    self.registers.set_half_carry_flag();

    self.set_last_clock(4);
  }

  // set bit b in register n
  pub fn set_b_n(&mut self, n: ByteRegister, b: u8) {
    let result = self.registers[n] | (1u8 << b);
    self.registers[n] = result;
    self.set_last_clock(2);
  }
  pub fn set_b_hlm(&mut self, b: u8) {
    let result = self.read_hl() | (1u8 << b);
    self.write_hl(result);
    self.set_last_clock(4);
  }

  // reset bit b in register n
  pub fn reset_set_b_n(&mut self, n: ByteRegister, b: u8) {
    let result = self.registers[n] & !(1u8 << b);
    self.registers[n] = result;
    self.set_last_clock(2);
  }
  pub fn reset_set_b_hlm(&mut self, b: u8) {
    let result = self.read_hl() & !(1u8 << b);
    self.write_hl(result);
    self.set_last_clock(4);
  }

  // ------------------------------------
  // Jumps
  // ------------------------------------

  // jump to address nn
  pub fn jp_nn(&mut self) {
    let address = self.next_word();
    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(3);
  }

  // jump if flag
  pub fn jp_nn_cc(&mut self, flag: Flag) {
    // TODO: How many cycles does this take if condition is not met?
    if !self.registers.get_flag(flag) { return; }

    // TODO: Is it correct that this doesn't take more cycles than jp_nn?!
    let address = self.next_word();
    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(3);
  }

  // jump if not flag
  pub fn jp_nn_ncc(&mut self, flag: Flag) {
    // TODO: How many cycles does this take if condition is not met?
    if self.registers.get_flag(flag) { return; }

    // TODO: Is it correct that this doesn't take more cycles than jp_nn?!
    let address = self.next_word();
    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(3);
  }

  // jump to address in HL
  pub fn jp_hl(&mut self) {
    let address = self.registers.read_word(WordRegister::HL);
    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(1);
  }

  // add n to current address and jump to it (relative jump)
  pub fn jr_n(&mut self) {
    let offset = self.next_byte() as u16;
    let address = self.registers
      .read_word(WordRegister::PC)
      .wrapping_add(offset);

    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(2);
  }

  // add n to current address and jump to it if flag
  pub fn jr_cc_n(&mut self, flag: Flag) {
    // TODO: How many cycles does this take if condition is not met?
    if !self.registers.get_flag(flag) { return; }

    let offset = self.next_byte() as u16;
    let address = self.registers
      .read_word(WordRegister::PC)
      .wrapping_add(offset);

    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(2);
  }

  // add n to current address and jump to it if not flag
  pub fn jr_ncc_n(&mut self, flag: Flag) {
    // TODO: How many cycles does this take if condition is not met?
    if self.registers.get_flag(flag) { return; }

    let offset = self.next_byte() as u16;
    let address = self.registers
      .read_word(WordRegister::PC)
      .wrapping_add(offset);

    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(2);
  }

  // ------------------------------------
  // Jumps
  // ------------------------------------

  // push address of next instruction onto stack and jump to nn
  pub fn call_nn(&mut self) {
    let address = self.next_word();
    let return_address = self.registers.read_word(WordRegister::PC);

    self.push_word(return_address);
    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(3);
  }

  // call if flag
  pub fn call_cc_nn(&mut self, flag: Flag) {
    // TODO: Cycles for early return
    if self.registers.get_flag(flag) { return; }

    let address = self.next_word();
    let return_address = self.registers.read_word(WordRegister::PC);

    self.push_word(return_address);
    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(3);
  }

  // call if not flag
  pub fn call_ncc_nn(&mut self, flag: Flag) {
    // TODO: Cycles for early return
    if !self.registers.get_flag(flag) { return; }

    let address = self.next_word();
    let return_address = self.registers.read_word(WordRegister::PC);

    self.push_word(return_address);
    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(3);
  }

  // ------------------------------------
  // Restarts
  // ------------------------------------

  // push current address and jump to 0x0 + n
  // TODO: Restrict these to possible values?
  pub fn rst_n(&mut self, n: u16) {
    let current = self.registers.read_word(WordRegister::PC);
    self.push_word(current);
    self.registers.write_word(WordRegister::PC, n);

    // damn this shit takes a while
    self.set_last_clock(8);
  }

  // ------------------------------------
  // Returns
  // ------------------------------------

  // pop two bytes and jump to address
  pub fn ret(&mut self) {
    let return_address = self.pop_word();
    self.registers.write_word(WordRegister::PC, return_address);
    self.set_last_clock(2);
  }

  // return if flag
  pub fn ret_cc(&mut self, flag: Flag) {
    // TODO: Cycles for early return
    if self.registers.get_flag(flag) { return; }

    let return_address = self.pop_word();
    self.registers.write_word(WordRegister::PC, return_address);
    self.set_last_clock(2);
  }

  // return if not flag
  pub fn ret_ncc(&mut self, flag: Flag) {
    // TODO: Cycles for early return
    if !self.registers.get_flag(flag) { return; }

    let return_address = self.pop_word();
    self.registers.write_word(WordRegister::PC, return_address);
    self.set_last_clock(2);
  }

  // return and enable interrupts
  pub fn reti(&mut self) {
    let return_address = self.pop_word();
    self.registers.write_word(WordRegister::PC, return_address);

    self.interrupts_enabled = true;

    self.set_last_clock(2);
  }

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

  fn set_flags_add8(&mut self, operands: (u8, u8)) {
    let result = (operands.0 as u16) + (operands.1 as u16);

    // TODO: Do this more efficiently
    if result == 0 { self.registers.set_zero_flag(); }
    if result > 255 { self.registers.set_carry_flag(); }

    if check_half_carry_add8(operands) {
      self.registers.set_half_carry_flag();
    }
  }

  fn set_flags_sub8(&mut self, operands: (u8, u8)) {
    let result = (operands.0 as i16) + (operands.1 as i16);

    self.registers.set_sub_flag();

    // TODO: Do this more efficiently
    if result == 0 { self.registers.set_zero_flag(); }
    if result < 0 { self.registers.set_carry_flag(); }

    if check_half_carry_sub8(operands) {
      self.registers.set_half_carry_flag();
    }
  }

  // TODO
  fn set_flags_add16(&mut self, operands: (u16, u16)) {}

  // TODO
  fn set_flags_sub16(&mut self, operands: (u16, u16)) {}

  // get byte at pc and increment pc
  fn next_byte(&mut self) -> u8 {
    let pc = self.registers.read_word(WordRegister::PC);
    let next_byte = self.read_pc();
    self.registers.write_word(WordRegister::PC, pc.wrapping_add(1));

    next_byte
  }

  fn next_word(&mut self) -> u16 {
    let first = self.next_byte() as u16;
    let second = self.next_byte() as u16;

    first | (second << 8)
  }

  fn push_byte(&mut self, byte: u8) {
    let current = self.registers.read_word(WordRegister::SP);
    let next = current.wrapping_sub(1);
    self.registers.write_word(WordRegister::SP, next);
    self.memory_interface.borrow_mut().write_byte(next, byte);
  }

  // TODO: Check if this is working correctly
  fn push_word(&mut self, word: u16) {
    let current = self.registers.read_word(WordRegister::SP);
    let next = current.wrapping_sub(2);
    self.registers.write_word(WordRegister::SP, next);
    self.memory_interface.borrow_mut().write_word(next, word);
  }

  fn pop_byte(&mut self) -> u8 {
    let current = self.registers.read_word(WordRegister::SP);
    let next = current.wrapping_add(1);
    self.registers.write_word(WordRegister::SP, next);

    self.memory_interface.borrow().read_byte(current)
  }

  // TODO: Check if this is working correctly
  fn pop_word(&mut self) -> u16 {
    let current = self.registers.read_word(WordRegister::SP);
    let next = current.wrapping_add(2);
    self.registers.write_word(WordRegister::SP, next);

    self.memory_interface.borrow().read_word(current)
  }
}

fn check_half_carry_add8(operands: (u8, u8)) -> bool {
  (((operands.0 & 0xF) + (operands.1 & 0xF)) & 0x10) == 0x10
}

fn check_half_carry_sub8(operands: (u8, u8)) -> bool {
  (((operands.0 as i16) & 0xF) - ((operands.1 as i16) & 0xF)) < 0
}

// TODO
fn check_carry_add16(operands: (u16, u16)) -> bool {
  true
}

// TODO
fn check_half_carry_add16(operands: (u16, u16)) -> bool {
  true
}

// TODO
fn check_carry_sub16(operands: (u16, u16)) -> bool {
  true
}

// TODO
fn check_half_carry_sub16(operands: (u16, u16)) -> bool {
  true
}
