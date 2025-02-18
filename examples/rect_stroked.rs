use impellers::*;

mod common;

fn main() {
    let framework = common::SdlGlImpellerFrameWork::new();
    // if you want to do any initialization before event loop,
    // this is the place for that.
    let dl = {
        let builder = DisplayListBuilder::new(None);
        let paint = Paint::default();
        paint.set_color(Color::BLACK); // clear with black first
        builder.draw_paint(&paint);
        paint.set_color(Color::ACID_GREEN);
        paint.set_draw_style(DrawStyle::Stroke);
        paint.set_stroke_width(15.0);
        builder.draw_rect(
            &Rect::new(Point::new(100.0, 100.0), Size::new(250.0, 250.0)),
            &paint,
        );
        builder.build().unwrap()
    };
    framework.enter_event_loop(Some(dl), None);
}
