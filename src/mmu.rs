pub trait MemoryInterface {
  fn read_byte(&self, addr: u16) -> u8;
  fn read_word(&self, addr: u16) -> u16;

  fn write_byte(&mut self, addr: u16, value: u8);
  fn write_word(&mut self, addr: u16, value: u16);
}

pub struct Memory {}

impl MemoryInterface for Memory {
  fn read_byte(&self, addr: u16) -> u8 { 0 }
  fn read_word(&self, addr: u16) -> u16 { 0 }

  fn write_byte(&mut self, addr: u16, value: u8) {}
  fn write_word(&mut self, addr: u16, value: u16) {}
}
