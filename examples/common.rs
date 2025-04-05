use glfw::*;
use glow::HasContext;
use impellers::*;

#[allow(unused)]
pub struct SdlGlImpellerFrameWork {
    pub ttx: TypographyContext,
    pub style: ParagraphStyle,
    pub itx: impellers::Context,
    pub glow_ctx: glow::Context,
    pub receiver: GlfwReceiver<(f64, WindowEvent)>,
    pub window: PWindow,
    pub gtx: Glfw,
    pub events: Vec<WindowEvent>,
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
        let mut gtx = init(fail_on_errors).expect("failed to init glfw");
        gtx.window_hint(WindowHint::ContextVersionMajor(4));
        gtx.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
        gtx.window_hint(WindowHint::SRgbCapable(true));
        gtx.window_hint(WindowHint::StencilBits(Some(8)));
        gtx.window_hint(WindowHint::ScaleToMonitor(true));

        let (mut window, ev_receiver) = gtx
            .create_window(800, 600, "glfw opengl impeller", WindowMode::Windowed)
            .unwrap();
        window.make_current();
        window.set_all_polling(true);

        // initialize impeller context using opengl fn pointers
        let itx = unsafe { impellers::Context::new_opengl_es(|s| window.get_proc_address(s) as _) }
            .unwrap();
        let glow_ctx =
            unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s) as _) };
        unsafe {
            let (width, height) = window.get_framebuffer_size();
            glow_ctx.viewport(0, 0, width, height);
        }
        let ttx = TypographyContext::default();
        let mut style = ParagraphStyle::default();
        style.set_font_size(24.0);
        style.set_font_family("Roboto");
        style.set_font_weight(FontWeight::Bold);
        let mut paint = Paint::default();
        paint.set_color(Color::LIGHT_SKY_BLUE);
        style.set_foreground(&paint);
        Self {
            ttx,
            style,
            glow_ctx,
            itx,
            window,
            receiver: ev_receiver,
            gtx,
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
        let mut previous_instant = std::time::Instant::now();
        let mut current_frame = 0;
        let mut fps = 0;
        let mut vsync = true;
        // enter event loop
        while !self.window.should_close() {
            {
                // record fps
                if previous_instant.elapsed().as_secs_f64() >= 1.0 {
                    fps = current_frame;
                    current_frame = 0;
                    previous_instant = std::time::Instant::now();
                }
                current_frame += 1;
            }
            self.events.clear();
            // check events
            self.gtx.poll_events();
            for (_, event) in flush_messages(&self.receiver) {
                match &event {
                    WindowEvent::Close => {
                        self.window.set_should_close(true);
                    }
                    WindowEvent::FramebufferSize(w, h) => {
                        unsafe {
                            self.glow_ctx.viewport(0, 0, *w, *h);
                        }
                        println!("window resized to {}x{}", w, h);
                    }
                    WindowEvent::Key(Key::Space, _, Action::Release, _) => {
                        vsync = !vsync;
                        println!("setting vsync to {vsync}");

                        self.gtx.set_swap_interval(if vsync {
                            SwapInterval::Sync(1)
                        } else {
                            SwapInterval::None
                        });
                    }
                    _ => {}
                }
                self.events.push(event);
            }

            let (width, height) = self.window.get_framebuffer_size();
            // init surface by wrapping default framebuffer (fbo = 0)
            let mut surface = unsafe {
                self.itx.wrap_fbo(
                    0,
                    PixelFormat::RGBA8888,
                    ISize::new(width.into(), height.into()),
                )
            }
            .expect("failed to wrap window's framebuffer");
            let mut dl_builder = DisplayListBuilder::new(Some(&Rect::from_size(
                [width as f32, height as f32].into(),
            )));

            if let Some(dl) = dl.as_ref() {
                dl_builder.draw_display_list(dl, 1.0);
            }
            // call user callback
            if let Some(cb) = user_callback.as_mut() {
                if let Some(display_list) = cb(&mut self) {
                    dl_builder.draw_display_list(&display_list, 1.0);
                }
            }
            {
                let mut para_builder = ParagraphBuilder::new(&self.ttx).unwrap();
                para_builder.push_style(&self.style);
                para_builder.add_text(&format!("avg fps: {fps}"));
                let para = para_builder.build(1000.0).unwrap();
                dl_builder.draw_paragraph(&para, Point::origin());
            }
            surface
                .draw_display_list(&dl_builder.build().unwrap())
                .unwrap();

            // submit frame and wait for vsync
            self.window.swap_buffers();
        }
    }

    // drop the window/sdl or whatever
}

#[allow(unused)]
pub fn main() {
    unimplemented!("this is a common module for other examples to use")
}
