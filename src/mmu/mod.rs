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
  
}

impl Memory {
  pub fn new() -> Memory {
    let ram = (0..=0x2000).map(|it| 0x00).collect::<Vec<u8>>();
    let vram = (0..=0x2000).map(|it| 0x00).collect::<Vec<u8>>();

    Memory {
      in_bios: true,

      ram,
      vram,
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
      0x8000...0x9FFF => { self.vram[address - 0x8000] },

      // RAM Bank n
      0xA000...0xBFFF => { 0 },

      // IRAM
      0xC000...0xDFFF => { 0 },

      // IRAM echo
      0xE000...0xFDFF => { 0 },

      // OAM
      0xFE00...0xFE9F => { 0 },

      // IO
      0xFF00...0xFF4B => { 0 },

      // register for unmapping the bootrom
      0xFF50 => { println!("bootrom unmapped!"); 0 },

      // zero page memory
      0xFF80...0xFFFE => { 0 },

      // interrupt enable register
      0xFFFF => { 0 },

      _ => { println!("invalid memory location: {:#x?}", addr); 0 }
    }
  }

  fn read_word(&self, addr: u16) -> u16 { 0 }

  fn write_byte(&mut self, addr: u16, value: u8) {}

  fn write_word(&mut self, addr: u16, value: u16) {}
}
