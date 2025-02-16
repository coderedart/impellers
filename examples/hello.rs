/*

use glfw::Context as _;
use impellers::*;
fn main() {
    // window creation
    let mut gtx = glfw::init_no_callbacks().unwrap(); // glfw context
    let (mut window, ev_receiver) = gtx
        .create_window(1280, 720, "Hello, World!", glfw::WindowMode::Windowed)
        .unwrap(); // glfw window and events receiver for that window's events
    window.set_all_polling(true); // which events we wanna receive
    window.make_current(); // make the context current
    let framebuffer_size = {
        // size of window's framebuffer/surface
        let framebuffer_size = window.get_framebuffer_size();
        ISize {
            width: framebuffer_size.0.into(),
            height: framebuffer_size.1.into(),
        }
    };

    // lets create impeller context
    let itx = unsafe {
        Context::new_opengl_es(|gl_fn_name| window.get_proc_address(gl_fn_name).cast_mut())
    }
    .expect("failed to create impeller context");
    // wrap the default framebuffer into a surface to draw on.
    let surface = unsafe {
        itx.wrap_fbo(0, PixelFormat::RGBA8888, &framebuffer_size)
            .expect("failed to create surface by wrapping default framebuffer")
    };

    // enter event loop
    while !window.should_close() {
        // check for events
        gtx.poll_events();
        // process events
        for (_event_time, _event) in glfw::flush_messages(&ev_receiver) {}

        // A display list is a collection of drawing commands that can be reused.
        // But we just create a new one every frame as we animate based on current time.
        let display_list = {
            // create a display list builder
            let builder = DisplayListBuilder::new(None);
            // paint controls the properties of draw commands
            let mut paint = Paint::default();
            // eg: lets set the color to black. So, any drawing command with this paint will use that color.
            paint.set_color(&Color {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
                color_space: ColorSpace::SRGB,
            });
            // fill the bounds with a color (^^that we set above)
            builder.draw_paint(&paint);
            let current_time = gtx.get_time(); // time in seconds since start of the program
                                               // lets set the color to a color that changes with time.
                                               // sin/cos/tan will always be in the range of -1 to 1, so lets use abs to keep it in between 0 and 1.
            paint.set_color(&Color {
                red: current_time.sin().abs() as _,
                green: current_time.cos().abs() as _,
                blue: current_time.tan().abs() as _,
                alpha: 1.0,
                color_space: ColorSpace::SRGB,
            });
            builder.draw_rect(
                &Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 200.0,
                    height: 200.0,
                },
                &paint,
            );
            // finish recording the drawing commands. This is only a "list" and we haven't drawn anything yet.
            builder.build().expect("failed to build a display_list")
        };
        // Now, draw the display_list on the surface. All the commands we recorded in the display_list will be drawn.
        // you can redraw the display_list multiple times to animate it on any number of surfaces.
        surface
            .draw_display_list(&display_list)
            .expect("failed to draw on surface");

        // tell window to show our surface on screen. and prepare for next frame.
        window.swap_buffers();
    }
}
*/
fn main() {
    // glfw won't compile for hours on github windows CI. So, we will use sdl2 for now.
}
