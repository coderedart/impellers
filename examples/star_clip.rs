mod common;

use impellers::*;
const IMAGE_BYTES: &[u8] = include_bytes!("dog.jpg");
fn main() {
    let framework = common::SdlGlImpellerFrameWork::new();
    let pixels = image::load_from_memory(IMAGE_BYTES).unwrap().to_rgba8();
    let (width, height) = pixels.dimensions();
    let contents = pixels.into_raw();
    let tex = framework
        .itx
        .create_texture_with_rgba8(&contents, width, height)
        .unwrap();
    let dl = {
        let builder = DisplayListBuilder::new(None);

        // the rect inside image that contains the dog.
        let dog_rect = Rect::new([700.0, 500.0].into(), [600.0, 500.0].into());
        // where on canvas we will be drawing the above rect's texture contents
        // always a good idea to keep the scaling proportional.
        // the width and height are 600x500 from src_rect, so, we use widthxheight of 300x250 for dst_rect
        // The scaling will be uniform. The reason for 250 height is that our star will also roughly be 256x256 sized.
        let dst_rect = Rect::from_size([300.0, 250.0].into());

        let path = {
            // build a star path
            let path_builder = PathBuilder::default();
            path_builder.move_to([128.0, 0.0].into());
            path_builder.line_to([168.0, 80.0].into());
            path_builder.line_to([256.0, 93.0].into());
            path_builder.line_to([192.0, 155.0].into());
            path_builder.line_to([207.0, 244.0].into());
            path_builder.line_to([128.0, 202.0].into());
            path_builder.line_to([49.0, 244.0].into());
            path_builder.line_to([64.0, 155.0].into());
            path_builder.line_to([0.0, 93.0].into());
            path_builder.line_to([88.0, 80.0].into());
            path_builder.line_to([128.0, 0.0].into());
            path_builder.close();
            path_builder.take_path_new(FillType::NonZero)
        };
        {
            // clear screen
            let paint = Paint::default();
            paint.set_color(Color::BLACK);
            builder.draw_paint(&paint);
        }
        // first, lets clip so that only contents *inside* will be visible.
        {
            // always a good idea to push a new stack and work there.
            // Then, pop after you are done drawing.
            builder.save();
            // intersect ensures that only the inside of the clip is drawn (if it overlaps with the clips down the stack)
            builder.clip_path(&path, ClipOperation::Intersect);
            builder.draw_texture_rect(&tex, &dog_rect, &dst_rect, TextureSampling::Linear, None);
            builder.restore(); // clip is gone here.
        }
        {
            builder.save();
            // lets move to right, so we don't draw over the previous contents.
            builder.translate(300.0, 0.0);
            // difference ensures that the clip is inverted and nothing will be drawn *inside*
            builder.clip_path(&path, ClipOperation::Difference);
            builder.draw_texture_rect(&tex, &dog_rect, &dst_rect, TextureSampling::Linear, None);
            builder.restore(); // clip and translate are popped off stack
        }
        builder.build().unwrap()
    };
    framework.enter_event_loop(Some(dl), None);
}
