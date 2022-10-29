#![allow(dead_code)]

use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;

use crate::util::{Vec3, AsRgb, Color};

const IMAGE_WIDTH: u32 = 256;
const IMAGE_HEIGHT: u32 = 256;

pub fn test_image() {
    let mut img: RgbImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    let bar = ProgressBar::new(IMAGE_HEIGHT as u64);

    for y in 0..img.height() {
        for x in 0..img.width() {
            let c = Color::new(
                x as f32 / (IMAGE_WIDTH - 1) as f32,
                y as f32 / (IMAGE_HEIGHT - 1) as f32,
                0.25,
            );

            img.put_pixel(x, IMAGE_HEIGHT - 1 - y, c.as_rgb());
        }
        bar.inc(1);
    }
    bar.finish();

    img.save("test_output.png").unwrap();
}

pub fn test_vectormath() {
    let mut v = Vec3::new(2., 3., 4.);
    let v2 = Vec3::new(1., 2., 3.);
    v[0] += 1.;
    v /= 2.;

    println!("{}, {}, {}", v[0], v[1], v[2]);
    println!("{}, {}, {}", v.x, v.y, v.z);
    println!("{}", v - v2);

    println!("{}", v.component_mul(&v2)); // v * v
    println!("{}", v.norm()); // length
    println!("{}", v.norm_squared()); // length_squared
    println!("{}", v.dot(&v2));
    println!("{}", v.normalize()); // unit_vector
}
