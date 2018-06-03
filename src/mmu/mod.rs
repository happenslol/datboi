mod bootrom;

pub trait MemoryInterface {
  fn read_byte(&self, addr: u16) -> u8;
  fn read_word(&self, addr: u16) -> u16;

  fn write_byte(&mut self, addr: u16, value: u8);
  fn write_word(&mut self, addr: u16, value: u16);
}

pub struct Memory {
  in_bios: bool,

  ram: Vec<u8>,
  vram: Vec<u8>,
  zero_page: Vec<u8>,
}

impl Memory {
  pub fn new() -> Memory {
    let ram = (0..=0x2000).map(|_| 0x00).collect::<Vec<u8>>();
    let vram = (0..=0x2000).map(|_| 0x00).collect::<Vec<u8>>();
    let zero_page = (0..=0x7F).map(|_| 0x00).collect::<Vec<u8>>();

    Memory {
      in_bios: true,

      ram,
      vram,
      zero_page,
    }
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
      0x8000...0x9FFF => self.vram[address - 0x8000],

      // RAM Bank n
      0xA000...0xBFFF => { 0 },

      // RAM
      0xC000...0xDFFF => self.ram[address - 0xC000],

      // IRAM echo
      0xE000...0xFDFF => { 0 },

      // OAM
      0xFE00...0xFE9F => { 0 },

      // IO
      0xFF00...0xFF4B => { 0 },

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
        to_word(self.vram[address - 0x8000], self.vram[address - 0x8000 + 1])
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
      0x8000...0x9FFF => self.vram[address - 0x8000] = value,

      // RAM Bank n
      0xA000...0xBFFF => {},

      // RAM
      0xC000...0xDFFF => self.ram[address - 0xC000] = value,

      // IRAM echo
      0xE000...0xFDFF => {},

      // OAM
      0xFE00...0xFE9F => {},

      // IO
      0xFF00...0xFF4B => {},

      // register for unmapping the bootrom
      0xFF50 => println!("bootrom unmapped!"),

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
        self.vram[address - 0x8000] = (value >> 8) as u8;
        self.vram[address - 0x8000 + 1] = value as u8;
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
