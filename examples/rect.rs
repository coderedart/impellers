use impellers::{Color, DisplayListBuilder, Paint, Point, Rect, Size};

mod common;

fn main() {
    let framework = common::SdlGlImpellerFrameWork::new();
    // if you want to do any initialization before event loop,
    // this is the place for that.
    let dl = {
        let mut builder = DisplayListBuilder::new(None);
        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);
        // clear with black first
        builder.draw_paint(&paint);
        paint.set_color(Color::AIR_FORCE_BLUE);
        builder.draw_rect(
            &Rect::new(Point::new(100.0, 100.0), Size::new(250.0, 250.0)),
            &paint,
        );
        builder.build().unwrap()
    };
    framework.enter_event_loop(Some(dl), None);
}
