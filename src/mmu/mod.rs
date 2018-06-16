use std::rc::Rc;
use std::cell::RefCell;

use ::gpu::Gpu;

use std::fs::File;
use std::io::prelude::*;

mod bootrom;

pub trait MemoryInterface {
  fn read_byte(&self, addr: u16) -> u8;
  fn read_word(&self, addr: u16) -> u16;

  fn write_byte(&mut self, addr: u16, value: u8);
  fn write_word(&mut self, addr: u16, value: u16);
}

#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
  VBlank,
  Timer,
  Serial,
  LcdStat,
  Joypad,
}

pub struct Memory {
  in_bios: bool,

  interrupts: u8,
  pub current_interrupt: Option<Interrupt>,
  interrupt_master_enable: bool,
  interrupt_enable: u8,

  ram: Vec<u8>,
  zero_page: Vec<u8>,

  gpu: Rc<RefCell<Gpu>>,
}

impl Memory {
  pub fn load_rom(&mut self) {
    let mut rom = File::open("../tetris.gb").expect("rom not found");
    let mut buf = Vec::new();
    let read = rom.read_to_end(&mut buf);
    println!("read {:?} bytes", read);

    self.ram = buf;
  }

  pub fn new(gpu: Rc<RefCell<Gpu>>) -> Memory {
    let mut ram = (0..=0x2000).map(|_| 0x00).collect::<Vec<u8>>();
    let zero_page = (0..=0x7F).map(|_| 0x00).collect::<Vec<u8>>();

    Memory {
      in_bios: true,

      interrupts: 0x00,
      current_interrupt: None,
      interrupt_master_enable: false,

      // all interrupts enabled
      interrupt_enable: 0x1F,

      ram,
      zero_page,
      gpu,
    }
  }

  pub fn step(&mut self) {
    if self.gpu.borrow().vblank_interrupt {
      self.gpu.borrow_mut().vblank_interrupt = false;
      self.set_interrupt(Interrupt::VBlank, true);
    }
  }

  fn set_interrupt(&mut self, interrupt: Interrupt, value: bool) {
    if value { self.current_interrupt = Some(interrupt); }

    match (interrupt, value) {
      (Interrupt::VBlank, true) => self.interrupts |= 0x01,
      (Interrupt::VBlank, false) => self.interrupts &= 0xFE,
      _ => {},
    };
  }
}

impl MemoryInterface for Memory {
  fn read_byte(&self, addr: u16) -> u8 {
    let address = addr as usize;

    match address {
      // ROM
      0x0000...0x7FFF => {
        if !self.in_bios || address > 0x100 { self.ram[address] }
        else { bootrom::BOOTROM[address] }
      },

      // VRAM
      0x8000...0x9FFF => self.gpu.borrow().vram[address - 0x8000],

      // RAM Bank n
      0xA000...0xBFFF => { 0 },

      // RAM
      0xC000...0xDFFF => self.ram[address - 0xC000],

      // IRAM echo
      0xE000...0xFDFF => { 0 },

      // OAM
      0xFE00...0xFE9F => { 0 },

      // IO
      0xFF00...0xFF4B => {
        match address {
          0xFF44 => self.gpu.borrow().line,

          _ => 0,
        }
      },

      // register for unmapping the bootrom
      0xFF50 => { println!("tried to read from bootmap unmap register"); 0 },

      // zero page memory
      0xFF80...0xFFFE => self.zero_page[address - 0xFF80],

      // interrupt enable register
      0xFFFF => { 0 },

      _ => { println!("invalid memory location: {:#x?}", addr); 0 }
    }
  }

  fn read_word(&self, addr: u16) -> u16 {
    let address = addr as usize;

    match address {
      // ROM
      0x0000...0x7FFF => {
        if !self.in_bios || address > 0x100 {
          to_word(self.ram[address], self.ram[address + 1])
        } else {
          to_word(
            bootrom::BOOTROM[address],
            bootrom::BOOTROM[address + 1]
          )
        }
      },

      // VRAM
      0x8000...0x9FFF => {
        let gpu = self.gpu.borrow();
        to_word(
          gpu.vram[address - 0x8000],
          gpu.vram[address - 0x8000 + 1]
        )
      },

      // RAM Bank n
      0xA000...0xBFFF => 0,

      // RAM
      0xC000...0xDFFF => {
        to_word(self.ram[address - 0xC000], self.ram[address - 0xC000 + 1])
      },

      // IRAM echo
      0xE000...0xFDFF => 0,

      // OAM
      0xFE00...0xFE9F => 0,

      // IO
      0xFF00...0xFF4B => 0,

      // register for unmapping the bootrom
      0xFF50 => { println!("tried to read from bootmap unmap register"); 0 },

      // zero page memory
      0xFF80...0xFFFE => {
        to_word(self.zero_page[address - 0xFF80], self.zero_page[address - 0xFF80 + 1])
      },

      // interrupt enable register
      0xFFFF => 0,

      _ => { println!("invalid memory location: {:#x?}", addr); 0 }
    }
  }

  fn write_byte(&mut self, addr: u16, value: u8) {
    let address = addr as usize;

    match address {
      // ROM
      0x0000...0x7FFF => {},

      // VRAM
      0x8000...0x9FFF => self.gpu.borrow_mut().vram[address - 0x8000] = value,

      // RAM Bank n
      0xA000...0xBFFF => {},

      // RAM
      0xC000...0xDFFF => self.ram[address - 0xC000] = value,

      // shadow RAM
      0xE000...0xFDFF => {},

      // OAM
      0xFE00...0xFE9F => {},

      // IO
      0xFF00...0xFF4B => {
        match address {
          0xFF40 => self.gpu.borrow_mut().set_lcd_control(value),

          _ => {},
        };
      },

      // register for unmapping the bootrom
      0xFF50 => {
        self.in_bios = false;
      },

      // zero page memory
      0xFF80...0xFFFE => self.zero_page[address - 0xFF80] = value,

      // interrupt enable register
      0xFFFF => {},

      _ => println!("invalid memory location: {:#x?}", addr),
    };
  }

  fn write_word(&mut self, addr: u16, value: u16) {
    let address = addr as usize;

    match address {
      // ROM
      0x0000...0x7FFF => {},

      // VRAM
      0x8000...0x9FFF => {
        let mut gpu = self.gpu.borrow_mut();
        gpu.vram[address - 0x8000] = (value >> 8) as u8;
        gpu.vram[address - 0x8000 + 1] = value as u8;
      }

      // RAM Bank n
      0xA000...0xBFFF => {},

      // RAM
      0xC000...0xDFFF => {
        self.ram[address - 0xC000] = (value >> 8) as u8;
        self.ram[address - 0xC000 + 1] = value as u8;
      },

      // IRAM echo
      0xE000...0xFDFF => {},

      // OAM
      0xFE00...0xFE9F => {},

      // IO
      0xFF00...0xFF4B => {},

      // register for unmapping the bootrom
      0xFF50 => println!("bootrom unmapped!"),

      // zero page memory
      0xFF80...0xFFFE => {
        self.zero_page[address - 0xFF80] = (value >> 8) as u8;
        self.zero_page[address - 0xFF80 + 1] = value as u8;
      },

      // interrupt enable register
      0xFFFF => {},

      _ => println!("invalid memory location: {:#x?}", addr),
    };
  }
}

fn to_word(lower: u8, upper: u8) -> u16 {
  ((lower as u16) << 8) | (upper as u16)
}
