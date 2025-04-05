use ash::vk::Handle;
use glfw::*;
use impellers::*;

pub fn main() {
    let mut gtx = init(fail_on_errors).unwrap();
    gtx.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
    gtx.window_hint(WindowHint::ScaleToMonitor(true));
    let (mut window, ev_receiver) = gtx
        .create_window(800, 600, "rust-glfw-impeller demo", WindowMode::Windowed)
        .expect("failed to create window");
    window.set_all_polling(true);

    // initialize impeller context by loading vulkan fn pointers
    let itx = unsafe {
        assert!(gtx.vulkan_supported());
        // create context using callback
        impellers::Context::new_vulkan(false, |vk_instance, vk_proc_name| {
            let proc_name = std::ffi::CStr::from_ptr(vk_proc_name).to_str().unwrap();
            gtx.get_instance_proc_address_raw(
                ash::vk::Instance::from_raw(vk_instance as _),
                proc_name,
            ) as _
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

    let mut vulkan_surface_khr: ash::vk::SurfaceKHR = ash::vk::SurfaceKHR::null();
    window
        .create_window_surface(
            ash::vk::Instance::from_raw(vk_info.vk_instance as _),
            std::ptr::null(),
            &raw mut vulkan_surface_khr,
        )
        .result()
        .expect("failed to create vk surface khr");
    assert!(!vulkan_surface_khr.is_null(), "surface pointer is null");
    let mut vk_swapchain =
        unsafe { itx.create_new_vulkan_swapchain(vulkan_surface_khr.as_raw() as _) }
            .expect("failed to create vk swapchain");

    // enter event loop
    while !window.should_close() {
        // check events
        gtx.poll_events();
        for (_, event) in flush_messages(&ev_receiver) {
            if event == WindowEvent::Close {
                window.set_should_close(true);
            }
        }

        let clear_display_list = {
            // create a display list builder
            let mut builder = DisplayListBuilder::new(None);
            // paint controls the properties of draw commands
            let mut paint = Paint::default();
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
        let animating_dl = {
            // create a display list builder
            let mut builder = DisplayListBuilder::new(None);
            // paint controls the properties of draw commands
            let mut paint = Paint::default();
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
        let oval_dl = {
            // create a display list builder
            let mut builder = DisplayListBuilder::new(None);
            // paint controls the properties of draw commands
            let mut paint = Paint::default();
            // eg: lets set the color to black. So, any drawing command with this paint will use that color.
            paint.set_color(Color::GRANNY_APPLE);
            builder.draw_oval(&Rect::from_size(Size::new(200.0, 200.0)), &paint);
            // finish recording the drawing commands. This is only a "list" and we haven't drawn anything yet.
            builder.build().expect("failed to build a display_list")
        };
        let main_display_list = {
            let mut builder = DisplayListBuilder::new(None);
            builder.draw_display_list(&clear_display_list, 1.0);
            builder.draw_display_list(&animating_dl, 1.0);
            builder.draw_display_list(&oval_dl, 1.0);
            builder.build().expect("failed to build a display_list")
        };
        let mut surface = vk_swapchain.acquire_next_surface_new().unwrap();
        // Now, draw the display_list on the surface. All the commands we recorded in the display_list will be drawn.
        // you can redraw the display_list multiple times to animate it on any number of surfaces.

        surface
            .draw_display_list(&main_display_list)
            .expect("failed to draw on surface");
        // submit frame and wait for vsync
        surface.present().unwrap();
    }

    // drop the window/sdl or whatever
}
