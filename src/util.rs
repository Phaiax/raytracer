use image::Rgb;
use nalgebra::Vector3;
pub type Vec3 = Vector3<f32>;
pub type Color = Vec3;
type Point3 = Vec3;

pub trait AsRgb {
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
