use std::cell::RefCell;
use std::ops::Neg;

use eframe::epaint::ColorImage;
use image::{Rgb, Rgba};
use indicatif::ProgressBar;
use nalgebra::Vector3;

use rand::distributions::Uniform;
use rand::prelude::{Distribution, Rng};

pub type Vec3 = Vector3<f64>;
pub type Color = Vec3;
pub type Point3 = Vec3;

pub trait AsRgb {
    fn as_rgb(self) -> Rgb<u8>;
    fn as_f64_rgba(self) -> Rgba<f64>;
    fn as_rgb_multisample(self, samples_per_pixel: u32) -> Rgb<u8>;
    fn as_rgba_multisample(self, samples_per_pixel: u32) -> Rgba<u8>;
}

impl AsRgb for Color {
    fn as_rgb(self) -> Rgb<u8> {
        Rgb([
            (self.x.sqrt().clamp(0.0, 0.999) * 256.0) as u8,
            (self.y.sqrt().clamp(0.0, 0.999) * 256.0) as u8,
            (self.z.sqrt().clamp(0.0, 0.999) * 256.0) as u8,
        ])
    }

    #[inline(always)]
    fn as_f64_rgba(self) -> Rgba<f64> {
        Rgba([self.x, self.y, self.z, 1.0])
    }

    fn as_rgb_multisample(self, samples_per_pixel: u32) -> Rgb<u8> {
        let scale = 1.0 / samples_per_pixel as f64;
        Rgb([
            ((self.x * scale).sqrt().clamp(0.0, 0.999) * 256.0) as u8,
            ((self.y * scale).sqrt().clamp(0.0, 0.999) * 256.0) as u8,
            ((self.z * scale).sqrt().clamp(0.0, 0.999) * 256.0) as u8,
        ])
    }

    fn as_rgba_multisample(self, samples_per_pixel: u32) -> Rgba<u8> {
        let scale = 1.0 / samples_per_pixel as f64;
        Rgba([
            ((self.x * scale).sqrt().clamp(0.0, 0.999) * 256.0) as u8,
            ((self.y * scale).sqrt().clamp(0.0, 0.999) * 256.0) as u8,
            ((self.z * scale).sqrt().clamp(0.0, 0.999) * 256.0) as u8,
            255,
        ])
    }
}

pub fn vec3_random<D: Distribution<f64>, R: Rng>(distr: &D, rng: &mut R) -> Vec3 {
    Vec3::new(distr.sample(rng), distr.sample(rng), distr.sample(rng))
}

/// Random vector of length 0..1
pub fn random_in_unit_sphere<R: Rng>(rng: &mut R) -> Vec3 {
    let dist_m1p1: Uniform<f64> = Uniform::new(-1.0, 1.0);
    loop {
        let p = vec3_random(&dist_m1p1, rng);
        if p.magnitude_squared() < 1. {
            return p;
        }
    }
}

/// Random vector of length 1
pub fn random_unit_vector<R: Rng>(rng: &mut R) -> Vec3 {
    random_in_unit_sphere(rng).normalize()
}

/// Random vector of length 0..1 with z=0
pub fn random_in_unit_disk<R: Rng>(rng: &mut R) -> Vec3 {
    let dist_m1p1: Uniform<f64> = Uniform::new(-1.0, 1.0);
    loop {
        let p = Vector3::new(dist_m1p1.sample(rng), dist_m1p1.sample(rng), 0.0);
        if p.magnitude_squared() < 1.0 {
            return p;
        }
    }
}

pub fn near_zero(vec: &Vec3) -> bool {
    let s = 1e-8;
    vec.x.abs() < s && vec.y.abs() < s && vec.z.abs() < s
}

pub struct Ray {
    pub orig: Point3,
    pub dir: Vec3,
}

impl Ray {
    pub fn new(orig: Point3, dir: Vec3) -> Ray {
        Ray { orig, dir }
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.orig + self.dir * t
    }

    pub fn origin(&self) -> Point3 {
        self.orig
    }

    pub fn direction(&self) -> Vec3 {
        self.dir
    }
}

/// Return reflection of v on surface with normal vector n
pub fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    v - 2.0 * v.dot(n) * n
}

pub fn refract(uv: &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = (-uv).dot(n).min(1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = (1.0 - r_out_perp.magnitude_squared()).abs().sqrt().neg() * n;
    r_out_perp + r_out_parallel
}

pub trait ProgressBarWrapper: Send + Sync {
    fn set_length(&self, len: u64);
    fn inc(&self, delta: u64, get_immediate_image: &dyn Fn() -> ColorImage);
    fn finish(&self);
}

impl ProgressBarWrapper for ProgressBar {
    fn set_length(&self, len: u64) {
        self.set_length(len);
    }

    fn inc(&self, delta: u64, _get_immediate_image: &dyn Fn() -> ColorImage) {
        self.inc(delta);
    }

    fn finish(&self) {
        self.finish();
    }
}
