const std = @import("std");
const sdl = @cImport({
    @cInclude("SDL3/SDL.h");
});

const emulator = @import("emulator.zig");

pub const SDLError = error{
    InitFailed,
    CreateWindowFailed,
    CreateRendererFailed,
    CreateTextureFailed,

    LockTextureFailed,
    RenderFailed,
    PresentFailed,
};

pub const SDLContext = struct {
    window: *sdl.SDL_Window,
    renderer: *sdl.SDL_Renderer,
    texture: *sdl.SDL_Texture,

    pub fn init() !SDLContext {
        if (!sdl.SDL_Init(sdl.SDL_INIT_VIDEO)) {
            std.debug.print("Failed to initialize SDL: {s}\n", .{sdl.SDL_GetError()});
            return SDLError.InitFailed;
        }
        errdefer sdl.SDL_Quit();

        const window = sdl.SDL_CreateWindow("here come dat boi", 640, 576, 0) orelse {
            std.debug.print("Failed to create window: {s}\n", .{sdl.SDL_GetError()});
            return SDLError.CreateWindowFailed;
        };
        errdefer sdl.SDL_DestroyWindow(window);

        const renderer = sdl.SDL_CreateRenderer(window, 0) orelse {
            std.debug.print("Failed to create renderer: {s}\n", .{sdl.SDL_GetError()});
            return SDLError.CreateRendererFailed;
        };
        errdefer sdl.SDL_DestroyRenderer(renderer);

        const texture = sdl.SDL_CreateTexture(
            renderer,
            sdl.SDL_PIXELFORMAT_RGB24,
            sdl.SDL_TEXTUREACCESS_STREAMING,
            emulator.screen_width,
            emulator.screen_height,
        );
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

        for (0..emulator.screen_height) |y| {
            for (0..emulator.screen_width) |x| {
                pixels[y * pitch + x * 3 + 0] = @intCast(y * 255 / emulator.screen_width); // r
                pixels[y * pitch + x * 3 + 1] = @intCast(x * 255 / emulator.screen_height); // g
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
