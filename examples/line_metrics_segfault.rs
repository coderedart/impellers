use impellers::*;
mod common;

fn main() {
    let framework = common::SdlGlImpellerFrameWork::new();
    framework.enter_event_loop(
        None,
        Some(Box::new(move |framework| {
            let mut dlb = DisplayListBuilder::new(None);
            let paragraph = {
                let mut pb = ParagraphBuilder::new(&framework.ttx).unwrap();
                pb.add_text("hello world");
                pb.build(600.0).unwrap()
            };
            let _ = paragraph.get_line_metrics().unwrap();
            dlb.draw_paragraph(&paragraph, Point::new(20.0, 50.0));
            Some(dlb.build().unwrap())
        })),
    );
}
