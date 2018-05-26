// allow unused crate wide for now
#![allow(dead_code, unused)]

use std::rc::Rc;
use std::cell::RefCell;

mod cpu;
mod mmu;

use cpu::CPU;
use mmu::Memory;

fn main() {
  // TODO: Find another way to let the CPU access the mmu
  // if Rc<RefCell> turns out to be too much overhead
  let memory = Rc::new(RefCell::new(Memory {}));
  let cpu = CPU::new(memory);
}
