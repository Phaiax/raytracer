use image::{ImageBuffer, Rgb, RgbImage};
use indicatif::ProgressBar;

const IMAGE_WIDTH: u32 = 256;
const IMAGE_HEIGHT: u32 = 256;

fn test_image() {
    let mut img: RgbImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    let bar = ProgressBar::new(IMAGE_HEIGHT as u64);

    for y in 0..img.height() {
        for x in 0..img.width() {
            let r = x as f32 / (IMAGE_WIDTH - 1) as f32;
            let g = (IMAGE_HEIGHT - y) as f32 / (IMAGE_HEIGHT - 1) as f32;
            let b = 0.25;

            img.put_pixel(
                x,
                y,
                Rgb([
                    (r * 255.999) as u8,
                    (g * 255.999) as u8,
                    (b * 255.999) as u8,
                ]),
            );
        }
        bar.inc(1);
    }
    bar.finish();

    img.save("test_output.png").unwrap();
}

fn main() {
    test_image();
}
