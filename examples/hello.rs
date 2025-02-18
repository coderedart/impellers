use glfw::*;
use impellers::*;

pub fn main() {
    let mut gtx = init(fail_on_errors).unwrap();
    gtx.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    gtx.window_hint(WindowHint::ContextVersionMajor(4));
    gtx.window_hint(WindowHint::SRgbCapable(true));
    gtx.window_hint(WindowHint::ScaleToMonitor(true));
    let (mut window, ev_receiver) = gtx
        .create_window(800, 600, "rust-glfw-impeller demo", WindowMode::Windowed)
        .expect("failed to create window");
    window.set_all_polling(true);
    window.make_current();

    // initialize impeller context using opengl fn pointers
    let itx =
        unsafe { impellers::Context::new_opengl_es(|s| window.get_proc_address(s) as _) }.unwrap();

    // enter event loop
    while !window.should_close() {
        // check events
        gtx.poll_events();
        for (_, event) in flush_messages(&ev_receiver) {
            match event {
                WindowEvent::Close => {
                    window.set_should_close(true);
                }
                WindowEvent::FramebufferSize(_, _) => {
                    // glViewPort here
                }
                _ => {}
            }
        }

        let (width, height) = window.get_framebuffer_size();
        let surface = unsafe {
            itx.wrap_fbo(
                0,
                PixelFormat::RGBA8888,
                ISize::new(width.into(), height.into()),
            )
        }
        .expect("failed to wrap window's framebuffer");
        // create a display list
        let display_list = {
            // create a display list builder
            let builder = DisplayListBuilder::new(None);
            // paint controls the properties of draw commands
            let paint = Paint::default();
            // eg: lets set the color to black. So, any drawing command with this paint will use that color.
            paint.set_color(Color::BLACKBERRY);
            // fill the bounds with a color (^^that we set above)
            builder.draw_paint(&paint);
            let current_time = gtx.get_time(); // time in seconds since start of the program
                                               // lets set the color to a color that changes with time.
                                               // sin/cos/tan will always be in the range of -1 to 1, so lets use abs to keep it in between 0 and 1.
            paint.set_color(Color::new_srgb(
                current_time.sin().abs() as _,
                current_time.cos().abs() as _,
                current_time.tan().abs() as _,
            ));
            builder.draw_rect(&Rect::from_size(Size::new(200.0, 200.0)), &paint);
            // finish recording the drawing commands. This is only a "list" and we haven't drawn anything yet.
            builder.build().expect("failed to build a display_list")
        };
        // Now, draw the display_list on the surface. All the commands we recorded in the display_list will be drawn.
        // you can redraw the display_list multiple times to animate it on any number of surfaces.
        surface
            .draw_display_list(&display_list)
            .expect("failed to draw on surface");
        // submit frame and wait for vsync
        window.swap_buffers();
    }

    // drop the window/sdl or whatever
}
