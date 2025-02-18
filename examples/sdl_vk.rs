use impellers::*;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
pub fn main() {
    // initialize window
    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("rust-sdl3-impeller-vk demo", 800, 600)
        .position_centered()
        .vulkan()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    // initialize impeller context by loading vulkan fn pointers
    let itx = unsafe {
        let vk_proc_addr_fn = video_subsystem.vulkan_get_proc_address_function().unwrap();
        // very unsafe transmute, but we YOLO
        type ProcAddrFnTy =
            fn(*mut std::os::raw::c_void, *const std::os::raw::c_char) -> *mut std::os::raw::c_void;
        let vk_proc_addr_fn =
            std::mem::transmute::<unsafe extern "C" fn(), ProcAddrFnTy>(vk_proc_addr_fn);

        // create context using callback
        Context::new_vulkan(false, |vk_instance, vk_proc_name| {
            vk_proc_addr_fn(vk_instance, vk_proc_name)
        })
    }
    .expect("failed to create impeller context");
    // lets see what's in here
    let vk_info = itx.get_vulkan_info().expect("failed to get vulkan info");
    dbg!(vk_info);
    assert!(
        !vk_info.vk_instance.is_null(),
        "instance pointer from vulkan info is null"
    );
    let vulkan_surface_khr = window
        .vulkan_create_surface(vk_info.vk_instance as _)
        .expect("failed to create vk surface khr");
    assert!(!vulkan_surface_khr.is_null(), "surface pointer is null");
    let vk_swapchain = unsafe { itx.create_new_vulkan_swapchain(vulkan_surface_khr as _) }
        .expect("failed to create vk swapchain");

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

        // create a display list
        let display_list = {
            // create a display list builder
            let builder = DisplayListBuilder::new(None);
            // paint controls the properties of draw commands
            let paint = Paint::default();
            // eg: lets set the color to black. So, any drawing command with this paint will use that color.
            paint.set_color(Color::BLACK);
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
            builder.draw_rect(&Rect::from_size([200.0, 200.0].into()), &paint);
            // finish recording the drawing commands. This is only a "list" and we haven't drawn anything yet.
            builder.build().expect("failed to build a display_list")
        };
        let surface = vk_swapchain.acquire_next_surface_new().unwrap();
        // Now, draw the display_list on the surface. All the commands we recorded in the display_list will be drawn.
        // you can redraw the display_list multiple times to animate it on any number of surfaces.
        surface
            .draw_display_list(&display_list)
            .expect("failed to draw on surface");
        // submit frame and wait for vsync
        surface.present().unwrap();
    }

    // drop the window/sdl or whatever
}
