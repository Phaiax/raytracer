use std::cell::RefCell;

use image::Rgb;
use nalgebra::Vector3;

use rand::prelude::{Rng, Distribution};
use rand::distributions::Uniform;

pub type Vec3 = Vector3<f32>;
pub type Color = Vec3;
pub type Point3 = Vec3;

pub trait AsRgb {
    fn as_rgb(self) -> Rgb<u8>;
    fn as_rgb_multisample(self, samples_per_pixel: u32) -> Rgb<u8>;
}

impl AsRgb for Color {
    fn as_rgb(self) -> Rgb<u8> {
        Rgb([
            (self.x * 255.999) as u8,
            (self.y * 255.999) as u8,
            (self.z * 255.999) as u8,
        ])
    }

    fn as_rgb_multisample(self, samples_per_pixel: u32) -> Rgb<u8> {
        let scale = 1.0 / samples_per_pixel as f32;
        Rgb([
            ((self.x * scale).clamp(0.0, 0.999) * 256.0) as u8,
            ((self.y * scale).clamp(0.0, 0.999) * 256.0) as u8,
            ((self.z * scale).clamp(0.0, 0.999) * 256.0) as u8,
        ])
    }
}

pub fn vec3_random<D: Distribution<f32>, R: Rng>(distr: &D, rng: &mut R) -> Vec3 {
    Vec3::new(
        distr.sample(rng),
        distr.sample(rng),
        distr.sample(rng),
    )
}

pub fn random_in_unit_sphere<R: Rng>(rng: &mut R) -> Vec3 {
    let dist_m1p1 : Uniform<f32> = Uniform::new(-1.0, 1.0);
    loop {
        let p = vec3_random(&dist_m1p1, rng);
        if p.magnitude_squared() < 1. {
            return p;
        }
    }
}

pub struct Ray {
    pub orig: Point3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(orig: Point3, dir: Vec3) -> Ray {
        Ray { orig, dir }
    }

    pub fn at(&self, t: f32) -> Point3 {
        self.orig + self.dir * t
    }

    pub fn origin(&self) -> Point3 {
        self.orig
    }

    pub fn direction(&self) -> Vec3 {
        self.dir
    }
}
