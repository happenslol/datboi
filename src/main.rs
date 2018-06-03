// allow unused crate wide for now
#![allow(dead_code, unused)]

use std::io;

use std::rc::Rc;
use std::cell::RefCell;

mod core;
mod mmu;

use core::cpu::CPU;
use mmu::Memory;

fn main() {
  // TODO: Find another way to let the CPU access the mmu
  // if Rc<RefCell> turns out to be too much overhead
  let memory = Rc::new(RefCell::new(Memory::new()));
  let mut cpu = CPU::new(memory);

  loop { cpu.step(); }
}
