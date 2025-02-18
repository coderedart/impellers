use glow::HasContext;
use impellers::*;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
#[allow(unused)]
pub struct SdlGlImpellerFrameWork {
    pub itx: Context,
    pub glow_ctx: glow::Context,
    pub gl_ctx: sdl3::video::GLContext,
    pub window: sdl3::video::Window,
    pub event_pump: sdl3::EventPump,
    pub events: Vec<Event>,
}
impl Default for SdlGlImpellerFrameWork {
    fn default() -> Self {
        Self::new()
    }
}
#[allow(unused)]
pub const DOG_BYTES: &[u8] = include_bytes!("dog.jpg");

type UserCallback = Box<dyn FnMut(&mut SdlGlImpellerFrameWork) -> Option<DisplayList>>;
impl SdlGlImpellerFrameWork {
    pub fn new() -> Self {
        // initialize window
        let sdl_context = sdl3::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_major_version(4);
        gl_attr.set_context_profile(sdl3::video::GLProfile::Core);
        gl_attr.set_framebuffer_srgb_compatible(true);
        gl_attr.set_stencil_size(8);
        // gl_attr.set_multisample_buffers(1);
        // gl_attr.set_multisample_samples(4);
        let window = video_subsystem
            .window("rust-sdl3-impeller demo", 800, 600)
            .position_centered()
            .opengl()
            .resizable()
            .build()
            .unwrap();

        let event_pump = sdl_context.event_pump().unwrap();
        let gl_ctx = window.gl_create_context().unwrap();
        window.gl_set_context_to_current().unwrap();

        // initialize impeller context using opengl fn pointers
        let itx = unsafe {
            Context::new_opengl_es(|s| {
                video_subsystem
                    .gl_get_proc_address(s)
                    .map(|p| p as *mut _)
                    .unwrap_or(std::ptr::null_mut())
            })
        }
        .unwrap();
        let glow_ctx = unsafe {
            glow::Context::from_loader_function(|s| {
                video_subsystem
                    .gl_get_proc_address(s)
                    .map(|p| p as *mut _)
                    .unwrap_or(std::ptr::null_mut())
            })
        };
        Self {
            glow_ctx,
            itx,
            gl_ctx,
            window,
            event_pump,
            events: vec![],
        }
    }
    /// Enters event loop and continues until window is closed or escape is pressed
    /// # Arguments
    /// - dl: If specified, this display list will be drawn on each frame
    /// - user_callback: If specified, this callback will be called on each frame
    ///   - If it returns a DisplayList, we will draw that on surface after callback.
    ///   - The callback has access to everything, so it can draw to surface on its own if it needs to.
    pub fn enter_event_loop(
        mut self,
        dl: Option<DisplayList>,
        mut user_callback: Option<UserCallback>,
    ) {
        // enter event loop
        loop {
            let mut quit = false;
            self.events.clear();
            // check events
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        quit = true;
                    }
                    ev => {
                        if let Event::Window {
                            win_event: sdl3::event::WindowEvent::Resized(w, h),
                            ..
                        } = &ev
                        {
                            unsafe {
                                self.glow_ctx.viewport(0, 0, *w, *h);
                            }
                            println!("window resized to {}x{}", w, h);
                        }
                        self.events.push(ev);
                    }
                }
            }
            if quit {
                break;
            }
            let (width, height) = self.window.size_in_pixels();
            // init surface by wrapping default framebuffer (fbo = 0)
            let surface = unsafe {
                self.itx.wrap_fbo(
                    0,
                    PixelFormat::RGBA8888,
                    ISize::new(width.into(), height.into()),
                )
            }
            .expect("failed to wrap window's framebuffer");
            if let Some(dl) = dl.as_ref() {
                surface.draw_display_list(dl).unwrap();
            }
            // call user callback
            if let Some(cb) = user_callback.as_mut() {
                if let Some(display_list) = cb(&mut self) {
                    surface.draw_display_list(&display_list).unwrap();
                }
            }

            // submit frame and wait for vsync
            self.window.gl_swap_window();
        }

        // drop the window/sdl or whatever
    }
}

#[allow(unused)]
pub fn main() {
    unimplemented!("this is a common module for other examples to use")
}
