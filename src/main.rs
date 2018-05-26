// allow unused crate wide for now
#![allow(dead_code, unused)]

mod cpu;

use cpu::CPU;

fn main() {
  let cpu = CPU::new();
}
