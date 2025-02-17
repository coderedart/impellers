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
        let blur_filter = MaskFilter::new_blur(BlurStyle::Normal, 4.0);
        paint.set_mask_filter(&blur_filter);

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
