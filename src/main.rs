// allow unused crate wide for now
#![allow(dead_code, unused)]

extern crate gl;
extern crate glutin;

use std::io;
use std::rc::Rc;
use std::cell::RefCell;

mod core;
mod mmu;
mod gpu;

mod gl_context;

use gpu::Gpu;
use mmu::Memory;
use core::cpu::{CPU, InterruptHandler};

use gl_context::{GlContext, Event};

fn main() {
  // TODO: Find another way to let the CPU access the mmu
  // if Rc<RefCell> turns out to be too much overhead
  let gpu = Rc::new(RefCell::new(Gpu::new()));
  let memory = Rc::new(RefCell::new(Memory::new(gpu.clone())));
  let mut cpu = CPU::new(memory.clone());

  let mut texture = (0..=(160 * 144 * 3))
    .map(|it| (it % 255) as u8).collect::<Vec<u8>>();

  let mut acc = 0;

  let mut ctx = GlContext::new();

  'main: loop {
    for event in ctx.next_events() {
      match event {
        Event::KeyEvent(scancode) => match scancode {
          1 => break 'main,
          57 => {
            acc += 1;

            texture = texture.iter().map(|it| it.wrapping_add(acc)).collect::<Vec<u8>>();
          },
          it => println!("{}", it),
        },

        Event::CloseRequest => break 'main,

        _ => {},
      };
    }

    ctx.render_frame(&texture);
  }

  // loop {
    // {
    //   let mut memory = memory.borrow_mut();
    //   if let Some(interrupt) = memory.current_interrupt {
    //     memory.current_interrupt = None;
    //     cpu.interrupt_queue.push_back(interrupt);
    //   }
    // }

  //   cpu.step();
  //   gpu.borrow_mut().step(cpu.last_clock.t);
  //   memory.borrow_mut().step();
  // }

}
