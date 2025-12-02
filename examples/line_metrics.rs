use impellers::*;
mod common;

fn main() {
    let framework = common::SdlGlImpellerFrameWork::new();
    // if you want to do any initialization before event loop,
    // this is the place for that.
    let mut paint = Paint::default(); // paint object to reuse
    let paragraph = {
        let ttx = TypographyContext::default(); // register any custom fonts if you want
        let mut puilder = ParagraphBuilder::new(&ttx).unwrap();
        let mut pstyle = ParagraphStyle::default();
        // you can set a custom font family if you want, but lets just use the system fonts
        pstyle.set_font_size(48.0);
        pstyle.set_font_weight(FontWeight::ExtraBold);
        paint.set_color(Color::BLUEBERRY);
        pstyle.set_foreground(&paint);
        puilder.push_style(&pstyle);
        puilder.add_text("HELLO EVERYONE");
        puilder.build(600.0).unwrap()
    };
    let metrics = paragraph.get_line_metrics().unwrap();
    dbg!(metrics.get_width(0));
    dbg!(metrics.get_height(0));
    dbg!(metrics.get_ascent(0));
    dbg!(metrics.get_descent(0));
    dbg!(metrics.get_baseline(0));
    dbg!(metrics.get_left(0));
    dbg!(metrics.get_unscaled_ascent(0));
    dbg!(metrics.get_code_unit_start_index_utf16(0));

    let dl = {
        let mut builder = DisplayListBuilder::new(None);
        paint.set_color(Color::BLACK); // clear with black first
        builder.draw_paint(&paint);
        builder.draw_paragraph(&paragraph, Point::new(100.0, 100.0));
        builder.build().unwrap()
    };
    framework.enter_event_loop(Some(dl), None);
}
