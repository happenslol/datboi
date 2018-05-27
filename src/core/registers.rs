use std::ops::{Index, IndexMut};

#[derive(PartialEq, Copy, Clone)]
pub enum ByteRegister {
  A, B, C, D, E, H, L,
}

#[derive(PartialEq, Copy, Clone)]
pub enum WordRegister {
  // combined registers
  AF, BC, DE, HL,

  // pointers
  PC, SP,
}

pub struct Registers {
  // 8 bit registers
  a: u8,
  b: u8,
  c: u8,
  d: u8,
  e: u8,
  h: u8,
  l: u8,

  // Flags
  f: u8,

  // 16 bit registers
  // program counter
  pc: u16,
  // stack pointer
  sp: u16,
}

impl Registers {
  pub fn new() -> Registers {
    Registers {
      a: 0, b: 0, c: 0, d: 0, e: 0,
      h: 0, l: 0, f: 0,

      pc: 0, sp: 0,
    }
  }

  pub fn read_word(&self, reg: WordRegister) -> u16 {
    match reg {
      WordRegister::AF => ((self.a as u16) << 8) | (self.f as u16),
      WordRegister::BC => ((self.b as u16) << 8) | (self.c as u16),
      WordRegister::DE => ((self.d as u16) << 8) | (self.e as u16),
      WordRegister::HL => ((self.h as u16) << 8) | (self.l as u16),

      WordRegister::SP => self.sp,
      WordRegister::PC => self.pc,
    }
  }

  pub fn write_word(&mut self, reg: WordRegister, word: u16) {
    match reg {
      WordRegister::AF => {
        self.a = word as u8;
        self.f = (word >> 8) as u8;
      },
      WordRegister::BC => {
        self.b = word as u8;
        self.c = (word >> 8) as u8;
      },
      WordRegister::DE => {
        self.d = word as u8;
        self.e = (word >> 8) as u8;
      },
      WordRegister::HL => {
        self.h = word as u8;
        self.l = (word >> 8) as u8;
      },

      WordRegister::SP => self.sp = word,
      WordRegister::PC => self.pc = word,
    };
  }

  pub fn advance_pc(&mut self, amount: u16) { self.pc = self.pc + amount; }

  // Flags manipulation
  // TODO: Inline these?
  pub fn clear_flags(&mut self) { self.f = 0; }
  pub fn set_zero_flag(&mut self) { self.f = self.f | 0x80; }
  pub fn unset_zero_flag(&mut self) { self.f = self.f & 0x70; }
  pub fn set_carry_flag(&mut self) { self.f = self.f | 0x10; }
  pub fn set_half_carry_flag(&mut self) { self.f = self.f | 0x20; }
  pub fn unset_half_carry_flag(&mut self) { self.f = self.f & 0xD0; }
  pub fn unset_carry_flag(&mut self) { self.f = self.f & 0xE0; }
  pub fn set_sub_flag(&mut self) { self.f = self.f | 0x40; }
  pub fn unset_sub_flag(&mut self) { self.f = self.f & 0xB0; }
}

impl Index<ByteRegister> for Registers {
  type Output = u8;

  fn index(&self, reg: ByteRegister) -> &u8 {
    match reg {
      ByteRegister::A => &self.a,
      ByteRegister::B => &self.b,
      ByteRegister::C => &self.c,
      ByteRegister::D => &self.d,
      ByteRegister::E => &self.e,
      ByteRegister::H => &self.h,
      ByteRegister::L => &self.l,
    }
  }
}

impl IndexMut<ByteRegister> for Registers {
  fn index_mut(&mut self, reg: ByteRegister) -> &mut u8 {
    match reg {
     ByteRegister::A => &mut self.a,
     ByteRegister::B => &mut self.b,
     ByteRegister::C => &mut self.c,
     ByteRegister::D => &mut self.d,
     ByteRegister::E => &mut self.e,
     ByteRegister::H => &mut self.h,
     ByteRegister::L => &mut self.l,
    }
  }
}
