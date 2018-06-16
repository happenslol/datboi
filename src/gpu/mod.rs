use std::io;
use std::io::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum GpuMode {
  // blanks
  HBlank,
  VBlank,

  // accessing ram
  ScanVRam,
  ScanOAM,
}

pub struct Gpu {
  pub mode: GpuMode,
  pub vram: Vec<u8>,

  pub vblank_interrupt: bool,

  clock: u32,
  pub line: u8,

  pub enabled: bool,

  pub back_buffer: Box<[u8; 32 * 32]>,
}

impl Gpu {
  pub fn new() -> Gpu {
    let mode = GpuMode::HBlank;

    // TODO: How big is this supposed to be?
    let vram = (0..=0x3000).map(|_| 0x99).collect::<Vec<u8>>();

    Gpu {
      mode,
      vram,

      vblank_interrupt: false,

      clock: 0,
      line: 0,

      enabled: false,
      back_buffer: Box::new([0; 32 * 32]),
    }
  }

  pub fn render_screen_to_texture(&self, texture: &mut Vec<u8>) {
    for (row_index, row) in self.back_buffer.chunks(32).enumerate() {
      let y_component = row_index * 8 * 256 * 3;

      for (tile_index, tile) in row.iter().enumerate() {
        let x_component = tile_index * 8 * 3;

        let vram_offset = (*tile as usize) * 16;
        let tile_row_indices = (vram_offset..(vram_offset + 16)).collect::<Vec<usize>>();
        for (tile_row_index, tile_row) in tile_row_indices.chunks(2).enumerate() {
          for i in 0..8 {
            let low = (self.vram[tile_row[0]] >> (7 - i)) & 1;
            let high = (self.vram[tile_row[1]] >> (7 - i)) & 1;
            let color = low | (high << 1);

            let tile_x = i * 3;
            let tile_y = tile_row_index * 256 * 3;

            let texture_index = (
              y_component +
              x_component +
              tile_x +
              tile_y
            );

            match color {
              0 => {
                texture[texture_index] = 230;
                texture[texture_index + 1] = 230;
                texture[texture_index + 2] = 230;
              },
              1 => {
                texture[texture_index] = 160;
                texture[texture_index + 1] = 160;
                texture[texture_index + 2] = 160;
              },
              2 => {
                texture[texture_index] = 80;
                texture[texture_index + 1] = 80;
                texture[texture_index + 2] = 80;
              },
              3 => {
                texture[texture_index] = 0;
                texture[texture_index + 1] = 0;
                texture[texture_index + 2] = 0;
              },
              it => println!("invalid color: {}", it),
            };
          }
        }
      }
    }
  }

  fn render_line(&mut self) {
    if self.line >= 32 { return; }

    let offset = (self.line as u16 * 32) as usize;
    let vram_offset = 0x1800 + offset;
    for (index_offset, i) in (vram_offset..(vram_offset + 32)).enumerate() {
      self.back_buffer[offset + index_offset] = self.vram[i];
    }
  }

  pub fn render_vram_to_texture(&self, texture: &mut Vec<u8>) {

    for (tile_index, tile) in self.vram.chunks(16).enumerate() {
      let texture_row = tile_index / 16;
      let texture_row_offset = texture_row * 128 * 3 * 7;

      for (row_index, row) in tile.chunks(2).enumerate() {
        for i in 0..=7 {
          let low = (row[0] >> (7 - i)) & 1;
          let high = (row[1] >> (7 - i)) & 1;
          let color = low | (high << 1);

          let y_component = (row_index * 128 * 3);
          let x_component = (i * 3);
          let tile_offset = (tile_index * 8 * 3);

          let texture_index = y_component + x_component + tile_offset + texture_row_offset;

          if texture_index + 3 >= texture.len() { break; }

          match color {
            0 => {
              texture[texture_index] = 230;
              texture[texture_index + 1] = 230;
              texture[texture_index + 2] = 230;
            },
            1 => {
              texture[texture_index] = 160;
              texture[texture_index + 1] = 160;
              texture[texture_index + 2] = 160;
            },
            2 => {
              texture[texture_index] = 80;
              texture[texture_index + 1] = 80;
              texture[texture_index + 2] = 80;
            },
            3 => {
              texture[texture_index] = 0;
              texture[texture_index + 1] = 0;
              texture[texture_index + 2] = 0;
            },
            it => println!("invalid color: {}", it),
          };
        }
      }

      if tile_index >= 384 { break; }
    }
  }

  pub fn step(&mut self, cycles: u32) {
    if !self.enabled { return; }

    let mode = self.mode;

    self.clock += cycles;

    match mode {
      GpuMode::HBlank => {
        if self.clock < 204 { return; }

        self.clock = 0;
        self.line += 1;

        if self.line == 143 {

          // render here!!

          self.mode = GpuMode::VBlank;
          return;
        }

        self.mode = GpuMode::ScanOAM;
      },

      GpuMode::VBlank => {
        if self.clock < 456 { return; }
        if self.clock == 456 { self.vblank_interrupt = true; }

        self.clock = 0;

        // we spend 10 lines in vblank, so
        // we need to keep track of the current
        // line here
        self.line += 1;

        if self.line > 153 {
          self.line = 0;
          self.mode = GpuMode::ScanOAM;
        }
      },

      GpuMode::ScanOAM => {
        if self.clock < 80 { return; }

        self.clock = 0;
        self.mode = GpuMode::ScanVRam;
      },

      GpuMode::ScanVRam => {
        if self.clock < 172 { return; }

        self.clock = 0;
        self.mode = GpuMode::HBlank;

        self.render_line();
      },
    };
  }

  pub fn set_lcd_control(&mut self, value: u8) {
    self.enabled = value & 0x80 != 0;
  }
}
