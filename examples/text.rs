use impellers::*;
mod common;

fn main() {
    let framework = common::SdlGlImpellerFrameWork::new();
    // if you want to do any initialization before event loop,
    // this is the place for that.
    let dl = {
        let builder = DisplayListBuilder::new(None);
        let paint = Paint::default();
        let paragraph = {
            let ttx = TypographyContext::default(); // register any custom fonts if you want
            let puilder = ParagraphBuilder::new(&ttx).unwrap();
            let pstyle = ParagraphStyle::default();
            // you can set a custom font family if you want, but lets just use the system fonts
            pstyle.set_font_size(48.0);
            pstyle.set_font_weight(FontWeight::ExtraBold);
            paint.set_color(Color::BLUEBERRY);
            pstyle.set_foreground(&paint);
            puilder.push_style(&pstyle);
            puilder.add_text("HELLO EVERYONE");
            puilder.build(600.0).unwrap()
        };

        paint.set_color(Color::BLACK); // clear with black first
        builder.draw_paint(&paint);
        builder.draw_paragraph(&paragraph, Point { x: 100.0, y: 100.0 });
        builder.build().unwrap()
    };
    framework.enter_event_loop(Some(dl), None);
}
