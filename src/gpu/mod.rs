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
}

impl Gpu {
  pub fn new() -> Gpu {
    let mode = GpuMode::HBlank;

    // TODO: How big is this supposed to be?
    let vram = (0..=0x2000).map(|_| 0x00).collect::<Vec<u8>>();

    Gpu {
      mode,
      vram,

      vblank_interrupt: false,

      clock: 0,
      line: 0,

      enabled: false,
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

        // write to buffer here
      },
    };
  }

  pub fn set_lcd_control(&mut self, value: u8) {
    self.enabled = value & 0x80 != 0;
    if self.enabled {
      println!("enabled!")
    }
  }
}
