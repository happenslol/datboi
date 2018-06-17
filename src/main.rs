// allow unused crate wide for now
#![allow(dead_code, unused)]

extern crate gl;
extern crate glutin;

use std::io;
use std::rc::Rc;
use std::cell::RefCell;
use std::u16;

mod core;
mod mmu;
mod gpu;

mod gl_context;

use gpu::Gpu;
use mmu::{Memory, MemoryInterface};
use core::cpu::{CPU, InterruptHandler};
use core::registers::WordRegister;
use core::instructions::{OPCODES, BIT_OPCODES};

use gl_context::{GlContext, Event};

use std::thread;
use std::time::Duration;
use std::io::prelude::*;

fn main() {
  // TODO: Find another way to let the CPU access the mmu
  // if Rc<RefCell> turns out to be too much overhead
  let gpu = Rc::new(RefCell::new(Gpu::new()));
  let memory = Rc::new(RefCell::new(Memory::new(gpu.clone())));
  let mut cpu = CPU::new(memory.clone());

  memory.borrow_mut().load_rom();

  let mut texture = (0..=(256 * 256 * 3))
    .map(|it| 255u8).collect::<Vec<u8>>();

  let mut ctx = GlContext::new();

  let mut should_step = true;
  let mut break_at = 0x00;

  'main: loop {
    for event in ctx.next_events() {
      match event {
        Event::KeyEvent(scancode) => match scancode {
          1 => break 'main,
          it => println!("{}", it),
        },

        Event::CloseRequest => break 'main,

        _ => {},
      };
    }

    // {
    //   let mut memory = memory.borrow_mut();
    //   if let Some(interrupt) = memory.current_interrupt {
    //     memory.current_interrupt = None;
    //     cpu.interrupt_queue.push_back(interrupt);
    //   }
    // }

    // let mut steps = 70_224i32;
    let mut steps = 100i32;

    while steps > 0 {
      let current = cpu.registers.read_word(WordRegister::PC);

      if should_step || current == break_at {
        println!("breaking at {:#04X}", current);
        let op = memory.borrow().read_byte(current);
        if op == 0xCB {
          let bitop = memory.borrow().read_byte(current + 1);
          println!("next instruction: {}", BIT_OPCODES[bitop as usize]);
        } else {
          println!("next instruction: {}", OPCODES[op as usize]);
        }

        'debug: loop {
          print!(">> ");
          io::stdout().flush().expect("couldn't flush");
          let mut input = String::new();
          io::stdin().read_line(&mut input).ok().expect("couldn't read in");

          match input.trim() {
            "c" | "cont" | "continue" | "" => {
              should_step = false;
              break_at = 0x00;
              break 'debug;
            },
            "r" | "regs" | "registers" => {
              println!("showing registers");
            },
            "n" | "next" => {
              should_step = true;
              break_at = 0x00;
              break 'debug;
            },
            "x" | "exit" => {
              break 'main;
            },

            "bgmap" => {
              println!("printing bg map data");
              let range = (0x9800..=0x9BFF).collect::<Vec<u16>>();
              for (row_index, row) in range.chunks(32).enumerate() {
                print!("line {} - ", row_index);
                for i in row {
                  let byte = memory.borrow().read_byte(*i);
                  print!("{} ", byte);
                  io::stdout().flush().expect("couldn't flush");
                }
                print!("\n");
                io::stdout().flush().expect("couldn't flush");
              }
            },

            it => {
              if it.starts_with("break ") {
                let parts = it.split("0x")
                  .map(|it| String::from(it))
                  .collect::<Vec<String>>();

                let num = u16::from_str_radix(&parts[1], 16)
                  .expect(&format!("invalid hex value: {}", parts[1]));
                break_at = num;
                should_step = false;

                println!("will break at {:#04X}", num);

                break 'debug;
              } else {
                println!("unknown command: {}", it)
              }
            },
          };
        }
      }

      cpu.step();
      gpu.borrow_mut().step(cpu.last_clock.t);
      memory.borrow_mut().step();

      if gpu.borrow().enabled {
        gpu.borrow().render_screen_to_texture(&mut texture);
        ctx.render_frame(&texture);
      }

      steps -= cpu.last_clock.t as i32;
    }

    thread::sleep(Duration::from_millis(10));
  }
}
