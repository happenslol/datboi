use std::rc::Rc;
use std::cell::RefCell;

use std::io;

use super::registers::{Registers, ByteRegister, WordRegister, Flag};
use ::mmu::MemoryInterface;

pub struct Clock {
  pub m: u32,
  pub t: u32,
}

pub enum InterruptHandler {
  VBlank = 0x40,
  LcdStat = 0x48,
  Timer = 0x50,
  Serial = 0x58,
  Joypad = 0x60,
}

pub struct CPU {
  clock: Clock,
  registers: Registers,

  interrupts_enabled: bool,
  interrupts_enabled_after_next: bool,
  in_standby: bool,
  should_power_down: bool,

  should_step: bool,

  // clock for last instruction
  // TODO: should this be here or in the registers?
  pub last_clock: Clock,

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
      should_step: false,
      memory_interface,
      last_clock,
    }
  }

  pub fn step(&mut self) {
    let pc = self.registers.read_word(WordRegister::PC);

    let instruction = self.read_pc();
    self.registers.advance_pc(1);

    match instruction {
      0x06 => self.ld_nn_n(ByteRegister::B),
      0x0E => self.ld_nn_n(ByteRegister::C),
      0x16 => self.ld_nn_n(ByteRegister::D),
      0x1E => self.ld_nn_n(ByteRegister::E),
      0x26 => self.ld_nn_n(ByteRegister::H),
      0x2E => self.ld_nn_n(ByteRegister::L),

      0x7F => self.ld_r1_r2(ByteRegister::A, ByteRegister::A),
      0x78 => self.ld_r1_r2(ByteRegister::A, ByteRegister::B),
      0x79 => self.ld_r1_r2(ByteRegister::A, ByteRegister::C),
      0x7A => self.ld_r1_r2(ByteRegister::A, ByteRegister::D),
      0x7B => self.ld_r1_r2(ByteRegister::A, ByteRegister::E),
      0x7C => self.ld_r1_r2(ByteRegister::A, ByteRegister::H),
      0x7D => self.ld_r1_r2(ByteRegister::A, ByteRegister::L),
      0x7E => self.ld_r1_hlm(ByteRegister::A),
      0x40 => self.ld_r1_r2(ByteRegister::B, ByteRegister::B),
      0x41 => self.ld_r1_r2(ByteRegister::B, ByteRegister::C),
      0x42 => self.ld_r1_r2(ByteRegister::B, ByteRegister::D),
      0x43 => self.ld_r1_r2(ByteRegister::B, ByteRegister::E),
      0x44 => self.ld_r1_r2(ByteRegister::B, ByteRegister::H),
      0x45 => self.ld_r1_r2(ByteRegister::B, ByteRegister::L),
      0x46 => self.ld_r1_hlm(ByteRegister::B),
      0x47 => self.ld_r1_r2(ByteRegister::B, ByteRegister::A),
      0x48 => self.ld_r1_r2(ByteRegister::C, ByteRegister::B),
      0x49 => self.ld_r1_r2(ByteRegister::C, ByteRegister::C),
      0x4A => self.ld_r1_r2(ByteRegister::C, ByteRegister::D),
      0x4B => self.ld_r1_r2(ByteRegister::C, ByteRegister::E),
      0x4C => self.ld_r1_r2(ByteRegister::C, ByteRegister::H),
      0x4D => self.ld_r1_r2(ByteRegister::C, ByteRegister::L),
      0x4E => self.ld_r1_hlm(ByteRegister::C),
      0x4F => self.ld_r1_r2(ByteRegister::C, ByteRegister::A),
      0x50 => self.ld_r1_r2(ByteRegister::D, ByteRegister::B),
      0x51 => self.ld_r1_r2(ByteRegister::D, ByteRegister::C),
      0x52 => self.ld_r1_r2(ByteRegister::D, ByteRegister::D),
      0x53 => self.ld_r1_r2(ByteRegister::D, ByteRegister::E),
      0x54 => self.ld_r1_r2(ByteRegister::D, ByteRegister::H),
      0x55 => self.ld_r1_r2(ByteRegister::D, ByteRegister::L),
      0x56 => self.ld_r1_hlm(ByteRegister::D),
      0x57 => self.ld_r1_r2(ByteRegister::D, ByteRegister::A),
      0x58 => self.ld_r1_r2(ByteRegister::E, ByteRegister::B),
      0x59 => self.ld_r1_r2(ByteRegister::E, ByteRegister::C),
      0x5A => self.ld_r1_r2(ByteRegister::E, ByteRegister::D),
      0x5B => self.ld_r1_r2(ByteRegister::E, ByteRegister::E),
      0x5C => self.ld_r1_r2(ByteRegister::E, ByteRegister::H),
      0x5D => self.ld_r1_r2(ByteRegister::E, ByteRegister::L),
      0x5E => self.ld_r1_hlm(ByteRegister::E),
      0x5F => self.ld_r1_r2(ByteRegister::E, ByteRegister::A),
      0x60 => self.ld_r1_r2(ByteRegister::H, ByteRegister::B),
      0x61 => self.ld_r1_r2(ByteRegister::H, ByteRegister::C),
      0x62 => self.ld_r1_r2(ByteRegister::H, ByteRegister::D),
      0x63 => self.ld_r1_r2(ByteRegister::H, ByteRegister::E),
      0x64 => self.ld_r1_r2(ByteRegister::H, ByteRegister::H),
      0x65 => self.ld_r1_r2(ByteRegister::H, ByteRegister::L),
      0x66 => self.ld_r1_hlm(ByteRegister::H),
      0x67 => self.ld_r1_r2(ByteRegister::H, ByteRegister::A),
      0x68 => self.ld_r1_r2(ByteRegister::L, ByteRegister::B),
      0x69 => self.ld_r1_r2(ByteRegister::L, ByteRegister::C),
      0x6A => self.ld_r1_r2(ByteRegister::L, ByteRegister::D),
      0x6B => self.ld_r1_r2(ByteRegister::L, ByteRegister::E),
      0x6C => self.ld_r1_r2(ByteRegister::L, ByteRegister::H),
      0x6D => self.ld_r1_r2(ByteRegister::L, ByteRegister::L),
      0x6E => self.ld_r1_hlm(ByteRegister::L),
      0x6F => self.ld_r1_r2(ByteRegister::L, ByteRegister::A),
      0x77 => self.ld_hlm_r1(ByteRegister::A),
      0x70 => self.ld_hlm_r1(ByteRegister::B),
      0x71 => self.ld_hlm_r1(ByteRegister::C),
      0x72 => self.ld_hlm_r1(ByteRegister::D),
      0x73 => self.ld_hlm_r1(ByteRegister::E),
      0x74 => self.ld_hlm_r1(ByteRegister::H),
      0x75 => self.ld_hlm_r1(ByteRegister::L),
      0x36 => self.ld_hlm_n(),

      0x7F => self.ld_a_r1(ByteRegister::A),
      0x78 => self.ld_a_r1(ByteRegister::B),
      0x79 => self.ld_a_r1(ByteRegister::C),
      0x7A => self.ld_a_r1(ByteRegister::D),
      0x7B => self.ld_a_r1(ByteRegister::E),
      0x7C => self.ld_a_r1(ByteRegister::H),
      0x7D => self.ld_a_r1(ByteRegister::L),
      0x0A => self.ld_a_m(WordRegister::BC),
      0x1A => self.ld_a_m(WordRegister::DE),
      0x7E => self.ld_a_m(WordRegister::HL),
      0xFA => self.ld_a_nn(),
      0x3E => self.ld_a_nb(),

      0xEA => self.ld_nnm_a(),

      0xF2 => self.ld_a_cm(),
      0xE2 => self.ld_cm_a(),
      0x3A => self.ldd_a_hlm(),
      0x32 => self.ldd_hlm_a(),
      0x2A => self.ldi_a_hlm(),
      0x22 => self.ldi_hlm_a(),

      0xE0 => self.ldh_nm_a(),
      0xF0 => self.ldh_a_nm(),

      0x01 => self.ld_n_nn(WordRegister::BC),
      0x11 => self.ld_n_nn(WordRegister::DE),
      0x21 => self.ld_n_nn(WordRegister::HL),
      0x31 => self.ld_n_nn(WordRegister::SP),

      0xF9 => self.ld_sp_hl(),
      0xF8 => self.ld_hl_sp_n(),

      0x08 => self.ld_nnm_sp(),

      0xF5 => self.push_nn(WordRegister::AF),
      0xC5 => self.push_nn(WordRegister::BC),
      0xD5 => self.push_nn(WordRegister::DE),
      0xE5 => self.push_nn(WordRegister::HL),

      0xF1 => self.pop_nn(WordRegister::AF),
      0xC1 => self.pop_nn(WordRegister::BC),
      0xD1 => self.pop_nn(WordRegister::DE),
      0xE1 => self.pop_nn(WordRegister::HL),

      0x87 => self.add_n(ByteRegister::A),
      0x80 => self.add_n(ByteRegister::B),
      0x81 => self.add_n(ByteRegister::C),
      0x82 => self.add_n(ByteRegister::D),
      0x83 => self.add_n(ByteRegister::E),
      0x84 => self.add_n(ByteRegister::H),
      0x85 => self.add_n(ByteRegister::L),
      0x86 => self.add_hlm(),
      0xC6 => self.add_nb(),

      0x8F => self.adc_n(ByteRegister::A),
      0x88 => self.adc_n(ByteRegister::B),
      0x89 => self.adc_n(ByteRegister::C),
      0x8A => self.adc_n(ByteRegister::D),
      0x8B => self.adc_n(ByteRegister::E),
      0x8C => self.adc_n(ByteRegister::H),
      0x8D => self.adc_n(ByteRegister::L),
      0x8E => self.adc_hlm(),
      0xCE => self.adc_nb(),

      0x97 => self.sub_n(ByteRegister::A),
      0x90 => self.sub_n(ByteRegister::B),
      0x91 => self.sub_n(ByteRegister::C),
      0x92 => self.sub_n(ByteRegister::D),
      0x93 => self.sub_n(ByteRegister::E),
      0x94 => self.sub_n(ByteRegister::H),
      0x95 => self.sub_n(ByteRegister::L),
      0x96 => self.sub_hlm(),
      0xD6 => self.sub_nb(),

      0x9F => self.sbc_a_n(ByteRegister::A),
      0x98 => self.sbc_a_n(ByteRegister::B),
      0x99 => self.sbc_a_n(ByteRegister::C),
      0x9A => self.sbc_a_n(ByteRegister::D),
      0x9B => self.sbc_a_n(ByteRegister::E),
      0x9C => self.sbc_a_n(ByteRegister::H),
      0x9D => self.sbc_a_n(ByteRegister::L),
      0x9E => self.sbc_a_hlm(),

      0xA7 => self.and_n(ByteRegister::A),
      0xA0 => self.and_n(ByteRegister::B),
      0xA1 => self.and_n(ByteRegister::C),
      0xA2 => self.and_n(ByteRegister::D),
      0xA3 => self.and_n(ByteRegister::E),
      0xA4 => self.and_n(ByteRegister::H),
      0xA5 => self.and_n(ByteRegister::L),
      0xA6 => self.and_hlm(),
      0xE6 => self.and_nb(),

      0xB7 => self.or_n(ByteRegister::A),
      0xB0 => self.or_n(ByteRegister::B),
      0xB1 => self.or_n(ByteRegister::C),
      0xB2 => self.or_n(ByteRegister::D),
      0xB3 => self.or_n(ByteRegister::E),
      0xB4 => self.or_n(ByteRegister::H),
      0xB5 => self.or_n(ByteRegister::L),
      0xB6 => self.or_hlm(),
      0xF6 => self.or_nb(),

      0xAF => self.xor_n(ByteRegister::A),
      0xA8 => self.xor_n(ByteRegister::B),
      0xA9 => self.xor_n(ByteRegister::C),
      0xAA => self.xor_n(ByteRegister::D),
      0xAB => self.xor_n(ByteRegister::E),
      0xAC => self.xor_n(ByteRegister::H),
      0xAD => self.xor_n(ByteRegister::L),
      0xAE => self.xor_hlm(),
      0xEE => self.xor_nb(),

      0xBF => self.cp_r1(ByteRegister::A),
      0xB8 => self.cp_r1(ByteRegister::B),
      0xB9 => self.cp_r1(ByteRegister::C),
      0xBA => self.cp_r1(ByteRegister::D),
      0xBB => self.cp_r1(ByteRegister::E),
      0xBC => self.cp_r1(ByteRegister::H),
      0xBD => self.cp_r1(ByteRegister::L),
      0xBE => self.cp_hlm(),
      0xFE => self.cp_nb(),

      0x3C => self.inc_n(ByteRegister::A),
      0x04 => self.inc_n(ByteRegister::B),
      0x0C => self.inc_n(ByteRegister::C),
      0x14 => self.inc_n(ByteRegister::D),
      0x1C => self.inc_n(ByteRegister::E),
      0x24 => self.inc_n(ByteRegister::H),
      0x2C => self.inc_n(ByteRegister::L),
      0x34 => self.inc_hlm(),

      0x3D => self.dec_n(ByteRegister::A),
      0x05 => self.dec_n(ByteRegister::B),
      0x0D => self.dec_n(ByteRegister::C),
      0x15 => self.dec_n(ByteRegister::D),
      0x1D => self.dec_n(ByteRegister::E),
      0x25 => self.dec_n(ByteRegister::H),
      0x2D => self.dec_n(ByteRegister::L),
      0x35 => self.dec_hlm(),

      0x09 => self.add_hl_n(WordRegister::BC),
      0x19 => self.add_hl_n(WordRegister::DE),
      0x29 => self.add_hl_n(WordRegister::HL),
      0x39 => self.add_hl_n(WordRegister::SP),

      0xE8 => self.add_sp_n(),

      0x03 => self.inc_nn(WordRegister::BC),
      0x13 => self.inc_nn(WordRegister::DE),
      0x23 => self.inc_nn(WordRegister::HL),
      0x33 => self.inc_nn(WordRegister::SP),

      0x0B => self.dec_nn(WordRegister::BC),
      0x1B => self.dec_nn(WordRegister::DE),
      0x2B => self.dec_nn(WordRegister::HL),
      0x3B => self.dec_nn(WordRegister::SP),

      0x27 => self.daa(),

      0x2F => self.cpl(),
      0x3F => self.ccf(),
      0x37 => self.scf(),

      0x00 => self.nop(),
      0x76 => self.halt(),
      0xF3 => self.di(),
      0xFB => self.ei(),

      0x07 => self.rlca(),
      0x17 => self.rla(),
      0x0F => self.rrca(),
      0x1F => self.rra(),

      0xC3 => self.jp_nn(),
      0xC2 => self.jp_nn_ncc(Flag::Zero),
      0xCA => self.jp_nn_cc(Flag::Zero),
      0xD2 => self.jp_nn_ncc(Flag::Carry),
      0xDA => self.jp_nn_cc(Flag::Carry),

      0xE9 => self.jp_hl(),

      0x18 => self.jr_n(),
      0x20 => self.jr_ncc_n(Flag::Zero),
      0x28 => self.jr_cc_n(Flag::Zero),
      0x30 => self.jr_ncc_n(Flag::Carry),
      0x38 => self.jr_cc_n(Flag::Carry),

      0xCD => self.call_nn(),
      0xC4 => self.call_ncc_nn(Flag::Zero),
      0xCC => self.call_cc_nn(Flag::Zero),
      0xD4 => self.call_ncc_nn(Flag::Carry),
      0xDC => self.call_cc_nn(Flag::Carry),

      0xC7 => self.rst_n(0x00),
      0xCF => self.rst_n(0x08),
      0xD7 => self.rst_n(0x10),
      0xDF => self.rst_n(0x18),
      0xE7 => self.rst_n(0x20),
      0xEF => self.rst_n(0x28),
      0xF7 => self.rst_n(0x30),
      0xFF => self.rst_n(0x38),

      0xC9 => self.ret(),
      0xC0 => self.ret_ncc(Flag::Zero),
      0xC8 => self.ret_cc(Flag::Zero),
      0xD0 => self.ret_ncc(Flag::Carry),
      0xD8 => self.ret_cc(Flag::Carry),

      0xD9 => self.reti(),

      // two byte instructions
      0xCB => {
        let next_byte = self.next_byte();
        match next_byte {
          0x37 => self.swap_n(ByteRegister::A),
          0x30 => self.swap_n(ByteRegister::B),
          0x31 => self.swap_n(ByteRegister::C),
          0x32 => self.swap_n(ByteRegister::D),
          0x33 => self.swap_n(ByteRegister::E),
          0x34 => self.swap_n(ByteRegister::H),
          0x35 => self.swap_n(ByteRegister::L),
          0x36 => self.swap_hlm(),

          0x07 => self.rlc_n(ByteRegister::A),
          0x00 => self.rlc_n(ByteRegister::B),
          0x01 => self.rlc_n(ByteRegister::C),
          0x02 => self.rlc_n(ByteRegister::D),
          0x03 => self.rlc_n(ByteRegister::E),
          0x04 => self.rlc_n(ByteRegister::H),
          0x05 => self.rlc_n(ByteRegister::L),
          0x06 => self.rlc_hlm(),

          0x17 => self.rl_n(ByteRegister::A),
          0x10 => self.rl_n(ByteRegister::B),
          0x11 => self.rl_n(ByteRegister::C),
          0x12 => self.rl_n(ByteRegister::D),
          0x13 => self.rl_n(ByteRegister::E),
          0x14 => self.rl_n(ByteRegister::H),
          0x15 => self.rl_n(ByteRegister::L),
          0x16 => self.rl_hlm(),

          0x0F => self.rlc_n(ByteRegister::A),
          0x08 => self.rlc_n(ByteRegister::B),
          0x09 => self.rlc_n(ByteRegister::C),
          0x0A => self.rlc_n(ByteRegister::D),
          0x0B => self.rlc_n(ByteRegister::E),
          0x0C => self.rlc_n(ByteRegister::H),
          0x0D => self.rlc_n(ByteRegister::L),
          0x0E => self.rlc_hlm(),

          0x17 => self.rl_n(ByteRegister::A),
          0x10 => self.rl_n(ByteRegister::B),
          0x11 => self.rl_n(ByteRegister::C),
          0x12 => self.rl_n(ByteRegister::D),
          0x13 => self.rl_n(ByteRegister::E),
          0x14 => self.rl_n(ByteRegister::H),
          0x15 => self.rl_n(ByteRegister::L),
          0x16 => self.rl_hlm(),

          0x0F => self.rrc_n(ByteRegister::A),
          0x08 => self.rrc_n(ByteRegister::B),
          0x09 => self.rrc_n(ByteRegister::C),
          0x0A => self.rrc_n(ByteRegister::D),
          0x0B => self.rrc_n(ByteRegister::E),
          0x0C => self.rrc_n(ByteRegister::H),
          0x0D => self.rrc_n(ByteRegister::L),
          0x0E => self.rrc_hlm(),

          0x1F => self.rr_n(ByteRegister::A),
          0x18 => self.rr_n(ByteRegister::B),
          0x19 => self.rr_n(ByteRegister::C),
          0x1A => self.rr_n(ByteRegister::D),
          0x1B => self.rr_n(ByteRegister::E),
          0x1C => self.rr_n(ByteRegister::H),
          0x1D => self.rr_n(ByteRegister::L),
          0x1E => self.rr_hlm(),

          0x27 => self.sla_n(ByteRegister::A),
          0x20 => self.sla_n(ByteRegister::B),
          0x21 => self.sla_n(ByteRegister::C),
          0x22 => self.sla_n(ByteRegister::D),
          0x23 => self.sla_n(ByteRegister::E),
          0x24 => self.sla_n(ByteRegister::H),
          0x25 => self.sla_n(ByteRegister::L),
          0x26 => self.sla_hlm(),

          0x2F => self.sra_n(ByteRegister::A),
          0x28 => self.sra_n(ByteRegister::B),
          0x29 => self.sra_n(ByteRegister::C),
          0x2A => self.sra_n(ByteRegister::D),
          0x2B => self.sra_n(ByteRegister::E),
          0x2C => self.sra_n(ByteRegister::H),
          0x2D => self.sra_n(ByteRegister::L),
          0x2E => self.sra_hlm(),

          0x3F => self.srl_n(ByteRegister::A),
          0x38 => self.srl_n(ByteRegister::B),
          0x39 => self.srl_n(ByteRegister::C),
          0x3A => self.srl_n(ByteRegister::D),
          0x3B => self.srl_n(ByteRegister::E),
          0x3C => self.srl_n(ByteRegister::H),
          0x3D => self.srl_n(ByteRegister::L),
          0x3E => self.srl_hlm(),

          0x40 => self.bit_b_n(ByteRegister::B, 0),
          0x41 => self.bit_b_n(ByteRegister::C, 0),
          0x42 => self.bit_b_n(ByteRegister::D, 0),
          0x43 => self.bit_b_n(ByteRegister::E, 0),
          0x44 => self.bit_b_n(ByteRegister::H, 0),
          0x45 => self.bit_b_n(ByteRegister::L, 0),
          0x46 => self.bit_b_hlm(0),
          0x47 => self.bit_b_n(ByteRegister::A, 0),
          0x48 => self.bit_b_n(ByteRegister::B, 1),
          0x49 => self.bit_b_n(ByteRegister::C, 1),
          0x4A => self.bit_b_n(ByteRegister::D, 1),
          0x4B => self.bit_b_n(ByteRegister::E, 1),
          0x4C => self.bit_b_n(ByteRegister::H, 1),
          0x4D => self.bit_b_n(ByteRegister::L, 1),
          0x4E => self.bit_b_hlm(1),
          0x4F => self.bit_b_n(ByteRegister::A, 1),

          0x50 => self.bit_b_n(ByteRegister::B, 2),
          0x51 => self.bit_b_n(ByteRegister::C, 2),
          0x52 => self.bit_b_n(ByteRegister::D, 2),
          0x53 => self.bit_b_n(ByteRegister::E, 2),
          0x54 => self.bit_b_n(ByteRegister::H, 2),
          0x55 => self.bit_b_n(ByteRegister::L, 2),
          0x56 => self.bit_b_hlm(2),
          0x57 => self.bit_b_n(ByteRegister::A, 2),
          0x58 => self.bit_b_n(ByteRegister::B, 3),
          0x59 => self.bit_b_n(ByteRegister::C, 3),
          0x5A => self.bit_b_n(ByteRegister::D, 3),
          0x5B => self.bit_b_n(ByteRegister::E, 3),
          0x5C => self.bit_b_n(ByteRegister::H, 3),
          0x5D => self.bit_b_n(ByteRegister::L, 3),
          0x5E => self.bit_b_hlm(3),
          0x5F => self.bit_b_n(ByteRegister::A, 3),

          0x60 => self.bit_b_n(ByteRegister::B, 4),
          0x61 => self.bit_b_n(ByteRegister::C, 4),
          0x62 => self.bit_b_n(ByteRegister::D, 4),
          0x63 => self.bit_b_n(ByteRegister::E, 4),
          0x64 => self.bit_b_n(ByteRegister::H, 4),
          0x65 => self.bit_b_n(ByteRegister::L, 4),
          0x66 => self.bit_b_hlm(4),
          0x67 => self.bit_b_n(ByteRegister::A, 4),
          0x68 => self.bit_b_n(ByteRegister::B, 5),
          0x69 => self.bit_b_n(ByteRegister::C, 5),
          0x6A => self.bit_b_n(ByteRegister::D, 5),
          0x6B => self.bit_b_n(ByteRegister::E, 5),
          0x6C => self.bit_b_n(ByteRegister::H, 5),
          0x6D => self.bit_b_n(ByteRegister::L, 5),
          0x6E => self.bit_b_hlm(5),
          0x6F => self.bit_b_n(ByteRegister::A, 5),

          0x70 => self.bit_b_n(ByteRegister::B, 6),
          0x71 => self.bit_b_n(ByteRegister::C, 6),
          0x72 => self.bit_b_n(ByteRegister::D, 6),
          0x73 => self.bit_b_n(ByteRegister::E, 6),
          0x74 => self.bit_b_n(ByteRegister::H, 6),
          0x75 => self.bit_b_n(ByteRegister::L, 6),
          0x76 => self.bit_b_hlm(6),
          0x77 => self.bit_b_n(ByteRegister::A, 6),
          0x78 => self.bit_b_n(ByteRegister::B, 7),
          0x79 => self.bit_b_n(ByteRegister::C, 7),
          0x7A => self.bit_b_n(ByteRegister::D, 7),
          0x7B => self.bit_b_n(ByteRegister::E, 7),
          0x7C => self.bit_b_n(ByteRegister::H, 7),
          0x7D => self.bit_b_n(ByteRegister::L, 7),
          0x7E => self.bit_b_hlm(7),
          0x7F => self.bit_b_n(ByteRegister::A, 7),

          0xC0 => self.set_b_n(ByteRegister::B, 0),
          0xC1 => self.set_b_n(ByteRegister::C, 0),
          0xC2 => self.set_b_n(ByteRegister::D, 0),
          0xC3 => self.set_b_n(ByteRegister::E, 0),
          0xC4 => self.set_b_n(ByteRegister::H, 0),
          0xC5 => self.set_b_n(ByteRegister::L, 0),
          0xC6 => self.set_b_hlm(0),
          0xC7 => self.set_b_n(ByteRegister::A, 0),
          0xC8 => self.set_b_n(ByteRegister::B, 1),
          0xC9 => self.set_b_n(ByteRegister::C, 1),
          0xCA => self.set_b_n(ByteRegister::D, 1),
          0xCB => self.set_b_n(ByteRegister::E, 1),
          0xCC => self.set_b_n(ByteRegister::H, 1),
          0xCD => self.set_b_n(ByteRegister::L, 1),
          0xCE => self.set_b_hlm(1),
          0xCF => self.set_b_n(ByteRegister::A, 1),

          0xD0 => self.set_b_n(ByteRegister::B, 2),
          0xD1 => self.set_b_n(ByteRegister::C, 2),
          0xD2 => self.set_b_n(ByteRegister::D, 2),
          0xD3 => self.set_b_n(ByteRegister::E, 2),
          0xD4 => self.set_b_n(ByteRegister::H, 2),
          0xD5 => self.set_b_n(ByteRegister::L, 2),
          0xD6 => self.set_b_hlm(2),
          0xD7 => self.set_b_n(ByteRegister::A, 2),
          0xD8 => self.set_b_n(ByteRegister::B, 3),
          0xD9 => self.set_b_n(ByteRegister::C, 3),
          0xDA => self.set_b_n(ByteRegister::D, 3),
          0xDB => self.set_b_n(ByteRegister::E, 3),
          0xDC => self.set_b_n(ByteRegister::H, 3),
          0xDD => self.set_b_n(ByteRegister::L, 3),
          0xDE => self.set_b_hlm(3),
          0xDF => self.set_b_n(ByteRegister::A, 3),

          0xE0 => self.set_b_n(ByteRegister::B, 4),
          0xE1 => self.set_b_n(ByteRegister::C, 4),
          0xE2 => self.set_b_n(ByteRegister::D, 4),
          0xE3 => self.set_b_n(ByteRegister::E, 4),
          0xE4 => self.set_b_n(ByteRegister::H, 4),
          0xE5 => self.set_b_n(ByteRegister::L, 4),
          0xE6 => self.set_b_hlm(4),
          0xE7 => self.set_b_n(ByteRegister::A, 4),
          0xE8 => self.set_b_n(ByteRegister::B, 5),
          0xE9 => self.set_b_n(ByteRegister::C, 5),
          0xEA => self.set_b_n(ByteRegister::D, 5),
          0xEB => self.set_b_n(ByteRegister::E, 5),
          0xEC => self.set_b_n(ByteRegister::H, 5),
          0xED => self.set_b_n(ByteRegister::L, 5),
          0xEE => self.set_b_hlm(5),
          0xEF => self.set_b_n(ByteRegister::A, 5),

          0xF0 => self.set_b_n(ByteRegister::B, 6),
          0xF1 => self.set_b_n(ByteRegister::C, 6),
          0xF2 => self.set_b_n(ByteRegister::D, 6),
          0xF3 => self.set_b_n(ByteRegister::E, 6),
          0xF4 => self.set_b_n(ByteRegister::H, 6),
          0xF5 => self.set_b_n(ByteRegister::L, 6),
          0xF6 => self.set_b_hlm(6),
          0xF7 => self.set_b_n(ByteRegister::A, 6),
          0xF8 => self.set_b_n(ByteRegister::B, 7),
          0xF9 => self.set_b_n(ByteRegister::C, 7),
          0xFA => self.set_b_n(ByteRegister::D, 7),
          0xFB => self.set_b_n(ByteRegister::E, 7),
          0xFC => self.set_b_n(ByteRegister::H, 7),
          0xFD => self.set_b_n(ByteRegister::L, 7),
          0xFE => self.set_b_hlm(7),
          0xFF => self.set_b_n(ByteRegister::A, 7),

          0x80 => self.res_b_n(ByteRegister::B, 0),
          0x81 => self.res_b_n(ByteRegister::C, 0),
          0x82 => self.res_b_n(ByteRegister::D, 0),
          0x83 => self.res_b_n(ByteRegister::E, 0),
          0x84 => self.res_b_n(ByteRegister::H, 0),
          0x85 => self.res_b_n(ByteRegister::L, 0),
          0x86 => self.res_b_hlm(0),
          0x87 => self.res_b_n(ByteRegister::A, 0),
          0x88 => self.res_b_n(ByteRegister::B, 1),
          0x89 => self.res_b_n(ByteRegister::C, 1),
          0x8A => self.res_b_n(ByteRegister::D, 1),
          0x8B => self.res_b_n(ByteRegister::E, 1),
          0x8C => self.res_b_n(ByteRegister::H, 1),
          0x8D => self.res_b_n(ByteRegister::L, 1),
          0x8E => self.res_b_hlm(1),
          0x8F => self.res_b_n(ByteRegister::A, 1),

          0x90 => self.res_b_n(ByteRegister::B, 2),
          0x91 => self.res_b_n(ByteRegister::C, 2),
          0x92 => self.res_b_n(ByteRegister::D, 2),
          0x93 => self.res_b_n(ByteRegister::E, 2),
          0x94 => self.res_b_n(ByteRegister::H, 2),
          0x95 => self.res_b_n(ByteRegister::L, 2),
          0x96 => self.res_b_hlm(2),
          0x97 => self.res_b_n(ByteRegister::A, 2),
          0x98 => self.res_b_n(ByteRegister::B, 3),
          0x99 => self.res_b_n(ByteRegister::C, 3),
          0x9A => self.res_b_n(ByteRegister::D, 3),
          0x9B => self.res_b_n(ByteRegister::E, 3),
          0x9C => self.res_b_n(ByteRegister::H, 3),
          0x9D => self.res_b_n(ByteRegister::L, 3),
          0x9E => self.res_b_hlm(3),
          0x9F => self.res_b_n(ByteRegister::A, 3),

          0xA0 => self.res_b_n(ByteRegister::B, 4),
          0xA1 => self.res_b_n(ByteRegister::C, 4),
          0xA2 => self.res_b_n(ByteRegister::D, 4),
          0xA3 => self.res_b_n(ByteRegister::E, 4),
          0xA4 => self.res_b_n(ByteRegister::H, 4),
          0xA5 => self.res_b_n(ByteRegister::L, 4),
          0xA6 => self.res_b_hlm(4),
          0xA7 => self.res_b_n(ByteRegister::A, 4),
          0xA8 => self.res_b_n(ByteRegister::B, 5),
          0xA9 => self.res_b_n(ByteRegister::C, 5),
          0xAA => self.res_b_n(ByteRegister::D, 5),
          0xAB => self.res_b_n(ByteRegister::E, 5),
          0xAC => self.res_b_n(ByteRegister::H, 5),
          0xAD => self.res_b_n(ByteRegister::L, 5),
          0xAE => self.res_b_hlm(5),
          0xAF => self.res_b_n(ByteRegister::A, 5),

          0xB0 => self.res_b_n(ByteRegister::B, 6),
          0xB1 => self.res_b_n(ByteRegister::C, 6),
          0xB2 => self.res_b_n(ByteRegister::D, 6),
          0xB3 => self.res_b_n(ByteRegister::E, 6),
          0xB4 => self.res_b_n(ByteRegister::H, 6),
          0xB5 => self.res_b_n(ByteRegister::L, 6),
          0xB6 => self.res_b_hlm(6),
          0xB7 => self.res_b_n(ByteRegister::A, 6),
          0xB8 => self.res_b_n(ByteRegister::B, 7),
          0xB9 => self.res_b_n(ByteRegister::C, 7),
          0xBA => self.res_b_n(ByteRegister::D, 7),
          0xBB => self.res_b_n(ByteRegister::E, 7),
          0xBC => self.res_b_n(ByteRegister::H, 7),
          0xBD => self.res_b_n(ByteRegister::L, 7),
          0xBE => self.res_b_hlm(7),
          0xBF => self.res_b_n(ByteRegister::A, 7),

          _ => println!("unkown 2 byte instruction: 0xCB {:#04X}", next_byte),
        };
      },

      0x10 => {
        let next_byte = self.next_byte();
        match next_byte {
          0x00 => self.stop(),

          _ => println!("unkown 2 byte instruction: 0x10 {:#04X}", next_byte),
        }
      }

      _ => println!("unknown instruction: {:#04X}", instruction),
    };

    match pc {
      0x0003 => println!("zeroing vram"),
      0x000C => println!("setting up audio"),
      0x001D => println!("setting up bg palette"),
      0x0021 => println!("loading logo data into vram"),
      0x0034 => println!("load 8 more bytes into vram"),
      0x0040 => println!("setup background tilemap"),
      0x0055 => println!("logo start!"),
      0x0080 => println!("playing sound"),

      // logo scroll routine
      0x0058 => {
        let a = self.registers[ByteRegister::A];
        println!("loaded scroll count into register, a is now {:#04X}", a);
        // self.should_step = true;
      },
      _ => {},
    };

    self.clock.m += self.last_clock.m;
    self.clock.t += self.last_clock.t;

    if self.should_step {
      println!("running instruction {:#04X}", instruction);
      io::stdin().read_line(&mut String::new()).expect("err");
    }

    if instruction == 0x00 {
      println!("hit noop");
      io::stdin().read_line(&mut String::new()).expect("error");
    }
  }

  // ------------------------------------
  // 8-bit loads
  // ------------------------------------

  // Put n into nn
  pub fn ld_nn_n(&mut self, dst: ByteRegister) {
    let next_byte = self.next_byte() as i8;
    self.registers[dst] = next_byte as u8;
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
  pub fn ld_hlm_n(&mut self) {
    let next_byte = self.next_byte();
    self.write_hl(next_byte);
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
  pub fn ld_a_nn(&mut self) {
    let address = self.next_word();
    let byte = self.memory_interface.borrow_mut().read_byte(address);
    self.registers[ByteRegister::A] = byte;
    self.set_last_clock(4);
  }
  pub fn ld_a_nb(&mut self) {
    let next_byte = self.next_byte();
    self.registers[ByteRegister::A] = next_byte;
    self.set_last_clock(2);
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
  pub fn ldh_a_nm(&mut self) {
    let pointer = 0xFF00 + (self.next_byte() as u16);
    self.registers[ByteRegister::A] = self.memory_interface.borrow().read_byte(pointer);
    self.set_last_clock(3);
  }

  // Put value at A into 0xFF00 + n
  pub fn ldh_nm_a(&mut self) {
    let byte = self.registers[ByteRegister::A];
    let pointer = 0xFF00 + (self.next_byte() as u16);
    self.memory_interface.borrow_mut().write_byte(pointer, byte);
    self.set_last_clock(3);
  }

  // put value from a into (nn)
  pub fn ld_nnm_a(&mut self) {
    let byte = self.registers[ByteRegister::A];
    let address = self.next_word();
    self.memory_interface.borrow_mut().write_byte(address, byte);
    self.set_last_clock(4);
  }

  // ------------------------------------
  // 16-bit loads
  // ------------------------------------

  // Put value nn into n
  pub fn ld_n_nn(&mut self, dst: WordRegister) {
    let next_word = self.next_word();
    self.registers.write_word(dst, next_word);
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
  pub fn ld_nnm_sp(&mut self) {
    let dst = self.next_word();
    let word = self.registers.read_word(WordRegister::SP);
    self.memory_interface.borrow_mut().write_word(dst, word);
    self.set_last_clock(5);
  }

  // Push register pair nn onto stack, decrease SP twice
  pub fn push_nn(&mut self, src: WordRegister) {
    let word = self.registers.read_word(src);
    self.push_word(word);
    self.set_last_clock(4);
  }

  // Pop word off stack into register pair nn, increment SP twice
  pub fn pop_nn(&mut self, dst: WordRegister) {
    let word = self.pop_word();
    self.registers.write_word(dst, word);
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
  pub fn add_nb(&mut self) {
    let next_byte = self.next_byte();
    let operands = (self.registers[ByteRegister::A], next_byte);

    self.registers.clear_flags();
    self.set_flags_add8(operands);

    self.registers[ByteRegister::A] = operands.0.wrapping_add(operands.1);
    self.set_last_clock(2);
  }

  // add n + carry to a
  pub fn adc_n(&mut self, src: ByteRegister) {
    let flag = self.registers.get_flag(Flag::Carry) as u8;
    let operands = (
      self.registers[ByteRegister::A],
      self.registers[src].wrapping_add(flag)
    );
    let result = operands.0.wrapping_add(operands.1);

    self.registers.clear_flags();
    self.set_flags_add8(operands);

    self.registers[ByteRegister::A] = result as u8;
    self.set_last_clock(1);
  }
  pub fn adc_hlm(&mut self) {
    let flag = self.registers.get_flag(Flag::Carry) as u8;
    let operands = (self.registers[ByteRegister::A], self.read_hl().wrapping_add(flag));

    self.registers.clear_flags();
    self.set_flags_add8(operands);

    let result = operands.0.wrapping_add(operands.1);
    self.set_last_clock(2);
  }
  pub fn adc_nb(&mut self) {
    let next_byte = self.next_byte();
    let flag = self.registers.get_flag(Flag::Carry) as u8;
    let operands = (self.registers[ByteRegister::A], next_byte.wrapping_add(flag));

    self.registers.clear_flags();
    self.set_flags_add8(operands);

    let result = operands.0.wrapping_add(operands.1);
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
  pub fn sub_nb(&mut self) {
    let next_byte = self.next_byte();
    let operands = (self.registers[ByteRegister::A], next_byte);

    self.registers.clear_flags();
    self.set_flags_sub8(operands);

    self.registers[ByteRegister::A] = operands.0.wrapping_sub(operands.1);
    self.set_last_clock(2);
  }

  // subtract n and carry flag from A
  pub fn sbc_a_n(&mut self, src: ByteRegister) {
    let flag = self.registers.get_flag(Flag::Carry) as u8;
    let operands = (
      self.registers[ByteRegister::A],
      self.registers[src].wrapping_sub(flag)
    );
    
    self.registers.clear_flags();
    self.set_flags_sub8(operands);

    self.registers[ByteRegister::A] = operands.0.wrapping_sub(operands.1);
    self.set_last_clock(1);
    self.set_last_clock(1);
  }
  pub fn sbc_a_hlm(&mut self) {
    let flag = self.registers.get_flag(Flag::Carry) as u8;
    let operands = (self.registers[ByteRegister::A], self.read_hl().wrapping_sub(flag));

    self.registers.clear_flags();
    self.set_flags_sub8(operands);

    self.registers[ByteRegister::A] = operands.0.wrapping_sub(operands.1);
    self.set_last_clock(2);
  }
  pub fn sbc_a_nb(&mut self) {
    let next_byte = self.next_byte();
    let flag = self.registers.get_flag(Flag::Carry) as u8;
    let operands = (self.registers[ByteRegister::A], next_byte.wrapping_sub(flag));

    self.registers.clear_flags();
    self.set_flags_sub8(operands);

    self.registers[ByteRegister::A] = operands.0.wrapping_sub(operands.1);
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
  pub fn and_nb(&mut self) {
    let next_byte = self.next_byte();
    let result = self.registers[ByteRegister::A] & next_byte;

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
  pub fn or_nb(&mut self) {
    let next_byte = self.next_byte();
    let result = self.registers[ByteRegister::A] | next_byte;

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
  pub fn xor_nb(&mut self) {
    let next_byte = self.next_byte();
    let result = self.registers[ByteRegister::A] ^ next_byte;

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
  pub fn cp_nb(&mut self) {
    let next_byte = self.next_byte();
    let operands = (self.registers[ByteRegister::A], next_byte);
    
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
    let operands = (self.registers[dst], 1u8);
    let result = operands.0.wrapping_sub(operands.1);

    if result == 255 { io::stdin().read_line(&mut String::new()).expect("err"); }

    self.registers.unset_sub_flag();
    if result == 0 { self.registers.set_zero_flag(); }
    else { self.registers.unset_zero_flag(); }
    if (check_half_carry_sub8(operands)) {
      self.registers.set_half_carry_flag();
    } else {
      self.registers.unset_half_carry_flag();
    }

    self.registers[dst] = result;
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

  // ------------------------------------
  // 16-bit ALU
  // ------------------------------------

  // add n to HL
  pub fn add_hl_n(&mut self, src: WordRegister) {
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
    let result = self.registers.read_word(dst).wrapping_add(1);
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
    self.set_last_clock(1);
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
    self.set_last_clock(1);
  }

  // rotate A right
  pub fn rrca(&mut self) {
    let a = self.registers[ByteRegister::A];

    let lsb = a & 1;

    self.registers[ByteRegister::A] = (a >> 1) | (lsb << 7);

    self.registers.clear_flags();
    if lsb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
    self.set_last_clock(1);
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
    self.set_last_clock(1);
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
    self.set_last_clock(2);
  }
  pub fn rlc_hlm(&mut self) {
    let a = self.read_hl();

    let msb = a >> 7;
    let result = (a << 1) | msb;

    self.write_hl(result);

    self.registers.clear_flags();
    if msb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
    self.set_last_clock(4);
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
    self.set_last_clock(2);
  }
  pub fn rl_hlm(&mut self) {
    let a = self.read_hl();

    let new_carry = (a >> 7) != 0;
    // TODO: Check if this is correct
    let old_carry = (self.registers.f & 0x10) >> 4;

    self.write_hl((a << 1) | old_carry);

    self.registers.clear_flags();
    if new_carry { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
    self.set_last_clock(4);
  }

  // rotate n right
  pub fn rrc_n(&mut self, n: ByteRegister) {
    let a = self.registers[n];

    let lsb = a & 1;

    self.registers[n] = (a >> 1) | (lsb << 7);

    self.registers.clear_flags();
    if lsb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
    self.set_last_clock(2);
  }
  pub fn rrc_hlm(&mut self) {
    let a = self.read_hl();

    let lsb = a & 1;

    self.write_hl((a >> 1) | (lsb << 7));

    self.registers.clear_flags();
    if lsb != 0 { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
    self.set_last_clock(4);
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
    self.set_last_clock(2);
  }
  pub fn rr_hlm(&mut self) {
    let a = self.read_hl();

    let new_carry = (a & 1) != 0;
    // TODO: Check if this is correct
    let old_carry = self.registers.get_flag(Flag::Carry) as u8;

    self.write_hl((a >> 1) | (old_carry << 7));

    self.registers.clear_flags();
    if new_carry { self.registers.set_carry_flag(); }
    else { self.registers.unset_carry_flag(); }
    self.set_last_clock(4);
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
  pub fn res_b_n(&mut self, n: ByteRegister, b: u8) {
    let result = self.registers[n] & !(1u8 << b);
    self.registers[n] = result;
    self.set_last_clock(2);
  }
  pub fn res_b_hlm(&mut self, b: u8) {
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
    let offset = self.next_byte() as i8;
    let current = self.registers.read_word(WordRegister::PC) as i16;
    let address = current.wrapping_add(offset as i16);

    // println!("\tjumping to {:#04X}", address);

    self.registers.write_word(WordRegister::PC, address as u16);
    self.set_last_clock(2);
  }

  // add n to current address and jump to it if flag
  pub fn jr_cc_n(&mut self, flag: Flag) {
    let offset = self.next_byte() as i8;

    // TODO: How many cycles does this take if condition is not met?
    if !self.registers.get_flag(flag) { return; }

    let current = self.registers.read_word(WordRegister::PC) as i16;
    let address = current.wrapping_add(offset as i16);

    // println!("\tjumping to {:#04X}", address);

    self.registers.write_word(WordRegister::PC, address as u16);
    self.set_last_clock(2);
  }

  // add n to current address and jump to it if not flag
  pub fn jr_ncc_n(&mut self, flag: Flag) {
    let offset = self.next_byte() as i8;

    // TODO: How many cycles does this take if condition is not met?
    if self.registers.get_flag(flag) { return; }

    let current = self.registers.read_word(WordRegister::PC) as i16;
    let address = current.wrapping_add(offset as i16);

    // println!("\tjumping to {:#04X}", address);

    self.registers.write_word(WordRegister::PC, address as u16);
    self.set_last_clock(2);
  }

  // ------------------------------------
  // Jumps
  // ------------------------------------

  // push address of next instruction onto stack and jump to nn
  pub fn call_nn(&mut self) {
    let address = self.next_word();
    let return_address = self.registers.read_word(WordRegister::PC);

    // println!("\tcalling to {:#04X}", address);

    self.push_word(return_address);
    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(3);
  }

  // call if flag
  pub fn call_cc_nn(&mut self, flag: Flag) {
    let address = self.next_word();

    // TODO: Cycles for early return
    if self.registers.get_flag(flag) { return; }

    let return_address = self.registers.read_word(WordRegister::PC);

    // println!("\tcalling to {:#04X}", address);

    self.push_word(return_address);
    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(3);
  }

  // call if not flag
  pub fn call_ncc_nn(&mut self, flag: Flag) {
    let address = self.next_word();

    // TODO: Cycles for early return
    if !self.registers.get_flag(flag) { return; }

    let return_address = self.registers.read_word(WordRegister::PC);

    // println!("\tcalling to {:#04X}", address);

    self.push_word(return_address);
    self.registers.write_word(WordRegister::PC, address);
    self.set_last_clock(3);
  }

  // ------------------------------------
  // Restarts
  // ------------------------------------

  // push current address and jump to 0x0 + n
  // TODO: Restrict these to possible values?
  pub fn rst_n(&mut self, n: u8) {
    let current = self.registers.read_word(WordRegister::PC);
    self.push_word(current);
    self.registers.write_word(WordRegister::PC, 0x0000 + (n as u16));

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
    // println!("\treturning to {:#04X}", return_address);
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

  fn read_sp(&self) -> u8 {
    let pointer = self.registers.read_word(WordRegister::SP);
    self.memory_interface.borrow().read_byte(pointer)
  }

  fn read_sp_word(&self) -> u16 {
    let pointer = self.registers.read_word(WordRegister::SP);
    let lower = self.memory_interface.borrow().read_byte(pointer);
    let upper = self.memory_interface.borrow().read_byte(pointer + 1);

    ((upper as u16) << 8) | (lower as u16)
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
    let result = (operands.0 as i16) - (operands.1 as i16);

    self.registers.set_sub_flag();

    // TODO: Do this more efficiently
    if result == 0 { self.registers.set_zero_flag(); }
    if result < 0 { self.registers.set_carry_flag(); }

    if check_half_carry_sub8(operands) {
      self.registers.set_half_carry_flag();
    }
  }

  // get byte at pc and increment pc
  fn next_byte(&mut self) -> u8 {
    let next_byte = self.read_pc();
    self.registers.advance_pc(1);

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

    let sp = self.registers.read_word(WordRegister::SP);
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

    let result = self.memory_interface.borrow().read_word(current);

    let sp = self.registers.read_word(WordRegister::SP);

    result
  }
}

fn check_half_carry_add8(operands: (u8, u8)) -> bool {
  (((operands.0 & 0xF) + (operands.1 & 0xF)) & 0x10) == 0x10
}

fn check_half_carry_sub8(operands: (u8, u8)) -> bool {
  (((operands.0 as i16) & 0xF) - ((operands.1 as i16) & 0xF)) < 0
}

fn check_carry_add16(operands: (u16, u16)) -> bool {
  let result = (operands.0 as u32) + (operands.1 as u32);
  (result >> 9) & 1 != 0
}

// TODO: Do this more efficiently
fn check_half_carry_add16(operands: (u16, u16)) -> bool {
  let lhs = (operands.0 >> 7) & 1;
  let rhs = (operands.1 >> 7) & 1;

  (lhs & rhs) != 0
}
