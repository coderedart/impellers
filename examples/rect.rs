use impellers::{Color, DisplayListBuilder, Paint, Rect};

mod common;

fn main() {
    let framework = common::SdlGlImpellerFrameWork::new();
    // if you want to do any initialization before event loop,
    // this is the place for that.
    let dl = {
        let builder = DisplayListBuilder::new(None);
        let paint = Paint::default();
        paint.set_color(Color::BLACK);
        // clear with black first
        builder.draw_paint(&paint);
        paint.set_color(Color::AIR_FORCE_BLUE);
        builder.draw_rect(
            &Rect {
                x: 100.0,
                y: 100.0,
                width: 250.0,
                height: 250.0,
            },
            &paint,
        );
        builder.build().unwrap()
    };
    framework.enter_event_loop(Some(dl), None);
}
