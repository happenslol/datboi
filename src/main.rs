// allow unused crate wide for now
#![allow(dead_code, unused)]

extern crate gl;
extern crate glutin;

use std::io;

use std::rc::Rc;
use std::cell::RefCell;

use glutin::GlContext;

mod core;
mod mmu;
mod gpu;

use gpu::Gpu;
use mmu::Memory;
use core::cpu::{CPU, InterruptHandler};

fn main() {
  // TODO: Find another way to let the CPU access the mmu
  // if Rc<RefCell> turns out to be too much overhead
  let gpu = Rc::new(RefCell::new(Gpu::new()));
  let memory = Rc::new(RefCell::new(Memory::new(gpu.clone())));
  let mut cpu = CPU::new(memory.clone());

  loop {
    let mut next_interrupt = None;

    {
      let mut memory = memory.borrow_mut();
      if let Some(interrupt) = memory.current_interrupt {
        println!("found {:?} Interrupt", interrupt);
        io::stdin().read_line(&mut String::new()).expect("err");
        memory.current_interrupt = None;

        next_interrupt = Some(interrupt);
      }
    }

    if let Some(handler) = next_interrupt {
      cpu.rst_n(InterruptHandler::VBlank as u8);
    } else { cpu.step(); }
    
    gpu.borrow_mut().step(cpu.last_clock.t);
    memory.borrow_mut().step();
  }

  let mut events_loop = glutin::EventsLoop::new();
  let window = glutin::WindowBuilder::new()
      .with_title("oh shit waddup")
      .with_dimensions(400, 400);

  let context = glutin::ContextBuilder::new()
      .with_vsync(true);

  let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

  unsafe { gl_window.make_current().unwrap(); }

  unsafe {
      gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
      gl::ClearColor(1.0, 1.0, 0.0, 1.0);
  }

  let mut running = true;
  'main: loop {
    events_loop.poll_events(|event| {
      match event {
        glutin::Event::WindowEvent { event, .. } => match event {
          glutin::WindowEvent::CloseRequested => running = false,
          glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
          _ => {},
        },
        _ => {}
      };
    });


    unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
    gl_window.swap_buffers().unwrap();

    if !running { break 'main; }
  }
}
