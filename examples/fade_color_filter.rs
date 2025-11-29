use impellers::*;
mod common;

fn main() {
    let framework = common::SdlGlImpellerFrameWork::new();

    framework.enter_event_loop(
        None,
        Some(Box::new(|_| {
            let mut builder = DisplayListBuilder::new(None);
            builder.scale(2.0, 2.0);
            builder.save();
            let mut paint = Paint::default();
            paint.set_color(Color::MAGENTA);
            let rect = Rect::new(Point::zero(), Size::new(800.0, 400.0));
            // paint.set_draw_style(DrawStyle::Stroke);
            builder.draw_rect(&rect, &paint);
            // use color-source to draw a fading rectangle
            let color_source = ColorSource::new_linear_gradient(
                Default::default(),
                [800.0, 0.0].into(),
                &[Color::UFO_GREEN, Color::TRANSPARENT],
                &[0.0, 1.0],
                TileMode::Repeat,
                None,
            );
            paint.set_color_source(&color_source);
            // paint.set_draw_style(DrawStyle::Fill);
            builder.draw_rect(&rect, &paint);
            builder.restore();
            builder.build()
        })),
    );
}
