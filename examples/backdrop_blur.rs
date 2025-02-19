mod common;
use impellers::*;

fn main() {
    let framework = common::SdlGlImpellerFrameWork::new();
    let (width, height) = framework.window.get_framebuffer_size();
    //  we resize the image so that it fits within our window
    let pixels = image::load_from_memory(common::DOG_BYTES)
        .unwrap()
        .resize(
            width as _,
            height as _,
            image::imageops::FilterType::Triangle,
        )
        .into_rgba8();

    let (width, height) = pixels.dimensions();
    let contents = pixels.into_raw();
    let tex = unsafe {
        framework
            .itx
            .create_texture_with_rgba8(&contents, width, height)
            .unwrap()
    };
    let dl = {
        let builder = DisplayListBuilder::new(None);
        let paint = Paint::default();
        builder.draw_texture(&tex, Point::zero(), TextureSampling::Linear, &paint);

        let rect = Rect::new(Point::zero(), Size::new(width as f32, height as f32 / 2.0));
        builder.clip_rect(&rect, ClipOperation::Intersect);
        // the backdrop will be blurred by this (within the clip area set above)
        builder.save_layer(
            &rect,
            None,
            Some(&ImageFilter::new_blur(8.0, 8.0, TileMode::Clamp)),
        );
        // no need to do anything
        builder.restore();
        builder.build().unwrap()
    };
    framework.enter_event_loop(Some(dl), None);
}
