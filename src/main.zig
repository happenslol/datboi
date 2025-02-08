const std = @import("std");
const sdl = @cImport({
    @cInclude("SDL3/SDL.h");
});

const w = 256;
const h = 256;

const SDLError = error{
    InitFailed,
    CreateWindowFailed,
    CreateRendererFailed,
    CreateTextureFailed,

    LockTextureFailed,
    RenderFailed,
    PresentFailed,
};

const Flags = packed struct {
    zero: bool = false,
    operation: bool = false,
    half_carry: bool = false,
    carry: bool = false,

    _padding: u4 = 0,
};

const CPU = struct {
    clock: struct { m: u32, t: u32 },

    regs: struct {
        // 8-bit registers
        a: u8,
        b: u8,
        c: u8,
        d: u8,
        e: u8,
        h: u8,
        l: u8,
        f: Flags,

        // 16-bit registers
        pc: u16,
        sp: u16,

        // Clock for last instruction
        m: u32,
        t: u32,
    },
};

const SDLContext = struct {
    window: *sdl.SDL_Window,
    renderer: *sdl.SDL_Renderer,
    texture: *sdl.SDL_Texture,

    pub fn init() !SDLContext {
        if (!sdl.SDL_Init(sdl.SDL_INIT_VIDEO)) {
            std.debug.print("Failed to initialize SDL: {s}\n", .{sdl.SDL_GetError()});
            return SDLError.InitFailed;
        }
        errdefer sdl.SDL_Quit();

        const window = sdl.SDL_CreateWindow("here come dat boi", 800, 800, 0) orelse {
            std.debug.print("Failed to create window: {s}\n", .{sdl.SDL_GetError()});
            return SDLError.CreateWindowFailed;
        };
        errdefer sdl.SDL_DestroyWindow(window);

        const renderer = sdl.SDL_CreateRenderer(window, 0) orelse {
            std.debug.print("Failed to create renderer: {s}\n", .{sdl.SDL_GetError()});
            return SDLError.CreateRendererFailed;
        };
        errdefer sdl.SDL_DestroyRenderer(renderer);

        const texture = sdl.SDL_CreateTexture(renderer, sdl.SDL_PIXELFORMAT_RGB24, sdl.SDL_TEXTUREACCESS_STREAMING, w, h);
        if (texture == null) {
            std.debug.print("Failed to create texture: {s}\n", .{sdl.SDL_GetError()});
            return SDLError.CreateTextureFailed;
        }

        var pitch_c: c_int = undefined;
        var pixels_c: ?*anyopaque = null;

        if (!sdl.SDL_LockTexture(texture, null, &pixels_c, &pitch_c)) {
            std.debug.print("Failed to lock texture: {s}\n", .{sdl.SDL_GetError()});
            return SDLError.LockTextureFailed;
        }

        const pitch: usize = @intCast(pitch_c);
        const pixels = @as([*]u8, @ptrCast(pixels_c));

        for (0..w) |y| {
            for (0..h) |x| {
                pixels[y * pitch + x * 3 + 0] = @intCast(y * 255 / w); // r
                pixels[y * pitch + x * 3 + 1] = @intCast(x * 255 / h); // g
                pixels[y * pitch + x * 3 + 2] = 0; // b
            }
        }

        sdl.SDL_UnlockTexture(texture);

        return .{
            .window = window,
            .renderer = renderer,
            .texture = texture,
        };
    }

    pub fn run(self: SDLContext) !void {
        var event: sdl.SDL_Event = undefined;
        while (true) {
            while (sdl.SDL_PollEvent(&event)) {
                switch (event.type) {
                    sdl.SDL_EVENT_QUIT => return,
                    sdl.SDL_EVENT_KEY_DOWN => {
                        switch (event.key.key) {
                            sdl.SDLK_ESCAPE => return,
                            else => {
                                std.debug.print("Key pressed: {s}\n", .{sdl.SDL_GetKeyName(event.key.key)});
                            },
                        }
                    },

                    else => {},
                }
            }

            _ = sdl.SDL_RenderTexture(self.renderer, self.texture, null, null);
            _ = sdl.SDL_RenderPresent(self.renderer);
            std.time.sleep(16 * std.time.ns_per_ms);
        }
    }
};

pub fn main() !void {
    const ctx = try SDLContext.init();
    try ctx.run();
}
