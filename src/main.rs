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

use core::cpu::CPU;
use mmu::Memory;

fn main() {
  // TODO: Find another way to let the CPU access the mmu
  // if Rc<RefCell> turns out to be too much overhead
  let memory = Rc::new(RefCell::new(Memory::new()));
  let mut cpu = CPU::new(memory);

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

    cpu.step();

    unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
    gl_window.swap_buffers().unwrap();

    if !running { break 'main; }
  }
}
