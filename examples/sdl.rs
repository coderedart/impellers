use impellers::*;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
pub fn main() {
    // initialize window
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_major_version(4);
    gl_attr.set_context_profile(sdl3::video::GLProfile::Core);
    let window = video_subsystem
        .window("rust-sdl3-impeller demo", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let _gl_ctx = window.gl_create_context().unwrap();
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

    // enter event loop
    loop {
        let mut quit = false;
        // check events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    quit = true;
                }
                _ => {}
            }
        }
        if quit {
            break;
        }

        let (width, height) = window.size_in_pixels();
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
            let current_time = sdl3::timer::ticks() as f64 / 1000.0; // time in seconds since start of the program
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
        window.gl_swap_window();
    }

    // drop the window/sdl or whatever
}
