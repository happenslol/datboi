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
  pub tiles: Box<[Tile; 384]>,

  pub vblank_interrupt: bool,

  clock: u32,
  pub line: u8,

  pub enabled: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct Tile {
  pub pixels: [u8; 16],
}

impl Tile {
  pub fn new() -> Tile {
    Tile { pixels: [0u8; 16] }
  }
}

impl Gpu {
  pub fn new() -> Gpu {
    let mode = GpuMode::HBlank;

    // TODO: How big is this supposed to be?
    let vram = (0..=0x2000).map(|_| 0x99).collect::<Vec<u8>>();

    Gpu {
      mode,
      vram,
      tiles: Box::new([Tile::new(); 384]),

      vblank_interrupt: false,

      clock: 0,
      line: 0,

      enabled: false,
    }
  }

  pub fn render_vram_to_texture(&self, texture: &mut Vec<u8>) {
    // let mut tile = (0..(8 * 2)).map(|_| 0u8).collect::<Vec<u8>>();

    // tile[1] |= 0x7E;
    // tile[2] |= 0x7E;
    // tile[4] |= 0x7;
    // tile[5] |= 0x7;
    // tile[6] |= 0xE0;
    // tile[7] |= 0xE0;
    // tile[8] |= 0x6;
    // tile[9] |= 0x60;
    // tile[12] |= 0x3F;
    // tile[13] |= 0x3F;
    // tile[14] |= 0xF;
    // tile[15] |= 0xF;

    let mut acc = 0;
    for (tile_index, tile) in self.vram.chunks(16).enumerate() {
      let texture_row = acc / 16;
      let texture_row_offset = texture_row * 128 * 3 * 8;

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

      acc += 1;
      if acc > 384 { break; }
    }

    // for (tile_index, tile) in self.tiles.iter().enumerate() {
    // for (tile_index, tile) in self.vram.chunks(16).enumerate() {
    //   if tile_index * 16 > 0x17FF { break; }

    //   // two bytes per row
    //   // for (row, pixel) in tile.pixels.chunks(2).enumerate() {
    //   for (row, row_pixels) in tile.chunks(2).enumerate() {

    //     for i in 0..=7 {
    //       let low = (row_pixels[0] >> i) & 1;
    //       let high = (row_pixels[1] >> i) & 1;

    //       let color = low | (high << 1);

    //       let tile_row = tile_index % 4;
    //       let y_offset = (tile_index / 4) * 256 * 3;

    //       let position = (row * 256 * 3) + (tile_row * 8 * 3) + (i * 3) + y_offset;
    //       if position >= texture.len() { break; }

    //       match color {
    //         0 => {
    //           texture[position] = 230;
    //           texture[position + 1] = 230;
    //           texture[position + 2] = 230;
    //         },
    //         1 => {
    //           texture[position] = 160;
    //           texture[position + 1] = 160;
    //           texture[position + 2] = 160;
    //         },
    //         2 => {
    //           texture[position] = 80;
    //           texture[position + 1] = 80;
    //           texture[position + 2] = 80;
    //         },
    //         3 => {
    //           texture[position] = 0;
    //           texture[position + 1] = 0;
    //           texture[position + 2] = 0;
    //         },
    //         it => println!("invalid color: {}", it),
    //       };
    //     }
    //   }
    // }
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

        // write to buffer here
      },
    };
  }

  pub fn set_lcd_control(&mut self, value: u8) {
    self.enabled = value & 0x80 != 0;
  }
}
