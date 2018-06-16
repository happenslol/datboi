use std::{mem, ptr, str};
use ::glutin;
use ::glutin::GlContext as GlutinContext;
use ::gl;

const WIDTH: u32 = 128;
const HEIGHT: u32 = 256;

pub struct GlContext {
  events_loop: glutin::EventsLoop,
  window: glutin::GlWindow,
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
  KeyEvent(u32),
  CloseRequest,
}

impl GlContext {
  pub fn new() -> GlContext {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("oh shit waddup")
        .with_dimensions(WIDTH * 3, HEIGHT * 3);

    let context = glutin::ContextBuilder::new()
        .with_vsync(true);

    let gl_window = glutin::GlWindow::new(window, context, &events_loop)
      .expect("failed to make gl window");

    unsafe {
      gl_window.make_current().expect("failed to make context current");

      gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
      gl::ClearColor(1.0, 1.0, 1.0, 1.0);

      let vs = gl::CreateShader(gl::VERTEX_SHADER);
      gl::ShaderSource(
        vs, 1,
        [VERTEX_SHADER.as_ptr() as *const _].as_ptr(),
        ptr::null()
      );
      gl::CompileShader(vs);

      let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
      gl::ShaderSource(
        fs, 1,
        [FRAGMENT_SHADER.as_ptr() as *const _].as_ptr(),
        ptr::null()
      );
      gl::CompileShader(fs);

      let program = gl::CreateProgram();
      gl::AttachShader(program, vs);
      gl::AttachShader(program, fs);
      gl::LinkProgram(program);
      gl::UseProgram(program);

      let mut status = gl::FALSE as gl::types::GLint;
      gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

      // Fail on error
      if status != (gl::TRUE as gl::types::GLint) {
        let mut len: gl::types::GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetProgramInfoLog(
          program,
          len,
          ptr::null_mut(),
          buf.as_mut_ptr() as *mut gl::types::GLchar,
        );
        panic!(
          "{}",
          str::from_utf8(&buf)
            .ok()
            .expect("ProgramInfoLog not valid utf8")
        );
      }

      gl::DeleteShader(vs);
      gl::DeleteShader(fs);

      let mut vb = mem::uninitialized();
      gl::GenBuffers(1, &mut vb);
      gl::BindBuffer(gl::ARRAY_BUFFER, vb);
      gl::BufferData(
        gl::ARRAY_BUFFER,
        (VERTEX_DATA.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
        VERTEX_DATA.as_ptr() as *const _,
        gl::STATIC_DRAW
      );

      let mut eb = mem::uninitialized();
      gl::GenBuffers(1, &mut eb);
      gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, eb);
      gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        (ELEMENT_DATA.len() * mem::size_of::<u32>()) as gl::types::GLsizeiptr,
        ELEMENT_DATA.as_ptr() as *const _,
        gl::STATIC_DRAW
      );

      gl::VertexAttribPointer(
        0 as gl::types::GLuint, 2,
        gl::FLOAT, gl::FALSE,
        (2 * mem::size_of::<f32>()) as gl::types::GLsizei,
        ptr::null()
      );

      gl::EnableVertexAttribArray(0 as gl::types::GLuint);

      let texture = (0..=(WIDTH * HEIGHT * 3))
        .map(|it| 100u8).collect::<Vec<u8>>();

      let mut tex = mem::uninitialized();
      gl::GenTextures(1, &mut tex);
      gl::BindTexture(gl::TEXTURE_2D, tex);
      gl::TexImage2D(
        gl::TEXTURE_2D, 0,
        gl::RGB as _, WIDTH as _, HEIGHT as _,
        0, gl::RGB, gl::UNSIGNED_BYTE,
        texture.as_ptr() as *const _
      );

      gl::ActiveTexture(gl::TEXTURE0);

      gl::TexParameteri(
        gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER,
        gl::NEAREST as _
      );
      gl::TexParameteri(
        gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER,
        gl::NEAREST as _
      );
    }

    GlContext {
      events_loop,
      window: gl_window,
    }
  }

  pub fn next_events(&mut self) -> Vec<Event> {
    let mut events = Vec::new();
    let mut resize: Option<(u32, u32)> = None;

    self.events_loop.poll_events(|event| {
      match event {
        glutin::Event::WindowEvent { event, .. } => match event {
          glutin::WindowEvent::KeyboardInput { input, .. } => {
            events.push(Event::KeyEvent(input.scancode));
          },

          glutin::WindowEvent::CloseRequested => events.push(Event::CloseRequest),
          glutin::WindowEvent::Resized(w, h) => resize = Some((w, h)),
          _ => {},
        },
        _ => {}
      };
    });

    if let Some((w, h)) = resize {
      unsafe { gl::Viewport(0, 0, w as _, h as _); }
      self.window.resize(w, h);
    }

    events
  }

  pub fn render_frame(&self, data: &[u8]) {
    unsafe {
      gl::Clear(gl::COLOR_BUFFER_BIT);

      gl::TexSubImage2D(
        gl::TEXTURE_2D, 0, 0, 0,
        WIDTH as _, HEIGHT as _, gl::RGB as _,
        gl::UNSIGNED_BYTE,
        data.as_ptr() as *const _
      );

      gl::DrawElements(
        gl::TRIANGLES, 6,
        gl::UNSIGNED_INT,
        ptr::null()
      );
    }

    self.window.swap_buffers().expect("failed to swap buffers");
  }
}

static VERTEX_SHADER: &'static [u8] = b"
#version 330 core
layout (location = 0) in vec2 pos;

out vec2 tex_coords;
out vec3 test_color;

void main() {
  gl_Position = vec4(pos, 0.0, 1.0);
  int tex_pos = gl_VertexID % 4;

  if (tex_pos == 0) {
      tex_coords = vec2(1.0, 0.0);
      test_color = vec3(1.0, 0.0, 0.0);
  } else if (tex_pos == 1) {
      tex_coords = vec2(1.0, 1.0);
      test_color = vec3(0.0, 1.0, 0.0);
  } else if (tex_pos == 2) {
      tex_coords = vec2(0.0, 1.0);
      test_color = vec3(0.0, 0.0, 1.0);
  } else {
      tex_coords = vec2(0.0, 0.0);
      test_color = vec3(1.0, 1.0, 1.0);
  }
}
";

static FRAGMENT_SHADER: &'static [u8] = b"
#version 330 core
out vec4 color;
uniform sampler2D tex;

in vec2 tex_coords;
in vec3 test_color;

void main() {
  // color = vec4(test_color, 1.0);
  color = texture(tex, tex_coords);
} 
";

static VERTEX_DATA: [f32; 8] = [
  0.9, 0.9,
  0.9, -0.9,
  -0.9, -0.9,
  -0.9, 0.9
];

static ELEMENT_DATA: [u32; 6] = [
  0, 1, 3,
  1, 2, 3
];
