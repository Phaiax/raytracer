use image::{ImageBuffer, Rgb, RgbImage};
use indicatif::ProgressBar;
use nalgebra::Vector3;

const IMAGE_WIDTH: u32 = 256;
const IMAGE_HEIGHT: u32 = 256;

type Vec3 = Vector3<f32>;
type Color = Vec3;
type Point3 = Vec3;

trait AsRgb {
    fn as_rgb(self) -> Rgb<u8>;
}

impl AsRgb for Color {
    fn as_rgb(self) -> Rgb<u8> {
        Rgb([
            (self.x * 255.999) as u8,
            (self.y * 255.999) as u8,
            (self.z * 255.999) as u8,
        ])
    }
}

fn test_image() {
    let mut img: RgbImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    let bar = ProgressBar::new(IMAGE_HEIGHT as u64);

    for y in 0..img.height() {
        for x in 0..img.width() {
            let c = Color::new(
                x as f32 / (IMAGE_WIDTH - 1) as f32,
                (IMAGE_HEIGHT - y) as f32 / (IMAGE_HEIGHT - 1) as f32,
                0.25,
            );

            img.put_pixel(x, y, c.as_rgb());
        }
        bar.inc(1);
    }
    bar.finish();

    img.save("test_output.png").unwrap();
}

#[allow(dead_code)]
fn test_vectormath() {
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

fn main() {
    test_image();
    // test_vectormath();
    
}
