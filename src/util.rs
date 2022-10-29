use std::cell::RefCell;

use image::Rgb;
use nalgebra::Vector3;
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
pub type Vec3 = Vector3<f32>;
pub type Color = Vec3;
pub type Point3 = Vec3;

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

thread_local! {
    pub static SMALL_RNG: RefCell<SmallRng> = RefCell::new(SmallRng::seed_from_u64(232008239771));
    pub static RN_DISTR: Uniform<f32> = Uniform::new(0.0, 1.0);
}

fn rand01() -> f32 {
    RN_DISTR.with(|rn_distr| {
        SMALL_RNG.with(|small_rng| {
            let mut small_rng = small_rng.borrow_mut();
            rn_distr.sample(&mut *small_rng)
        })
    })
}
