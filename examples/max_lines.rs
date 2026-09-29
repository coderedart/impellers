use impellers::*;
mod common;
const LONG_STR: &str = "
This is first line.
This is second line. xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx.
This is third line. xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx.
";
fn main() {
    let framework = common::SdlGlImpellerFrameWork::new();
    // if you want to do any initialization before event loop,
    // this is the place for that.
    let dl = {
        let mut builder = DisplayListBuilder::new(None);
        let mut paint = Paint::default();
        paint.set_color(Color::BLACK); // clear with black first
        let mut text_paint = Paint::default();
        text_paint.set_color(Color::WHITE); // for text
        let mut used_height = 50.0;

        builder.draw_paint(&paint);

        let p = ellpsis_without_max_lines(&framework.ttx, &text_paint);
        builder.draw_paragraph(&p, Point::new(20.0, used_height));
        used_height += p.get_height() + 10.0;
        let p = ellipsis_with_max_lines(&framework.ttx, &text_paint);
        builder.draw_paragraph(&p, Point::new(20.0, used_height));
        used_height += p.get_height() + 10.0;
        let p = ellipsis_with_max_line_1(&framework.ttx, &text_paint);
        builder.draw_paragraph(&p, Point::new(20.0, used_height));
        dbg!(p.get_height(), p.get_line_count());

        builder.build().unwrap()
    };
    framework.enter_event_loop(Some(dl), None);
}

/// When max lines is not provided and ellipsis is set,
/// everything after the *first* overflowing line will be cut off.
/// In this example, we set ellipsis to "...".
/// The first line will fit inside width.
/// The second line will overflow and ellipsis will be added. This is the end.
/// Everything beyond (part of second line that got cut off) + third line is skipped.
fn ellpsis_without_max_lines(ttx: &TypographyContext, text_paint: &Paint) -> Paragraph {
    let mut puilder = ParagraphBuilder::new(ttx).unwrap();

    let mut pstyle = ParagraphStyle::default();
    // you can set a custom font family if you want, but lets just use the system fonts
    pstyle
        .set_font_size(24.0)
        .set_font_weight(FontWeight::ExtraBold)
        .set_ellipsis(Some("..."))
        .set_foreground(&text_paint);
    puilder.push_style(&pstyle);
    puilder.add_text(LONG_STR);
    puilder.build(300.0).unwrap()
}
/// When we provide both ellipsis and max lines,
/// we will wrap the overflowing lines until we hit the max lines.
/// In this example, we set max lines to 4 and ellipses to "...".
/// The first line will be consumed by first line.
/// The second line will consume 3 lines and still overflow, so, ellipsis will be added to it.
/// Everything beyond (part of second line that got cut off) + the following lines (third line) will be ignored.
/// 
fn ellipsis_with_max_lines(ttx: &TypographyContext, text_paint: &Paint) -> Paragraph {
    let mut puilder = ParagraphBuilder::new(ttx).unwrap();

    let mut pstyle = ParagraphStyle::default();
    pstyle
        .set_font_size(24.0)
        .set_font_weight(FontWeight::ExtraBold)
        .set_ellipsis(Some("..."))
        .set_max_lines(4)
        .set_foreground(&text_paint);
    puilder.push_style(&pstyle);
    puilder.add_text(LONG_STR);
    puilder.build(300.0).unwrap()
}

fn ellipsis_with_max_line_1(ttx: &TypographyContext, text_paint: &Paint) -> Paragraph {
    let mut puilder = ParagraphBuilder::new(ttx).unwrap();

    let mut pstyle = ParagraphStyle::default();
    // you can set a custom font family if you want, but lets just use the system fonts
    pstyle
        .set_font_size(24.0)
        .set_font_weight(FontWeight::ExtraBold)
        .set_ellipsis(Some("..."))
        .set_max_lines(2)
        .set_foreground(&text_paint);
    puilder.push_style(&pstyle);
    puilder.add_text(LONG_STR);
    puilder.build(300.0).unwrap()
}