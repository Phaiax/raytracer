//!
//! # Coordinate system
//!
//! Right Hand Coordinate system: Y to top, X to right
//! Camera is at origin (0,0,0), looking into -z direction.
//!
//! Viewport:
//!   - u goes to the right (x direction)
//!   - v goes to the top (y direction)
//!
//!

#![allow(dead_code, unused_imports)]

mod camera;
mod hittables;
mod playground;
mod util;
mod world;

use std::rc::Rc;

use crate::camera::Camera;
use crate::hittables::{Hittable, Sphere};
use crate::util::{random_in_unit_sphere, AsRgb, Color, Point3, Ray, Vec3};
use crate::world::World;
use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: u32 = 400;
const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32;
const SAMPLES_PER_PIXEL: u32 = 100;
const MAX_DEPTH: u32 = 50;

pub fn raytracer() {
    let mut img: RgbImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    let bar = ProgressBar::new(IMAGE_HEIGHT as u64);
    let mut small_rng = SmallRng::seed_from_u64(232008239771);
    let rn_distr: Uniform<f64> = Uniform::new(0.0, 1.0);

    // World

    let mut world = World::new();
    world.add(Sphere::new(0., 0., -1., 0.5));
    world.add(Sphere::new(0., -100.5, -1., 100.));

    let camera = Camera::new(ASPECT_RATIO);

    for y in 0..img.height() {
        for x in 0..img.width() {
            let mut c = Color::zeros();
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (x as f64 + rn_distr.sample(&mut small_rng)) / (IMAGE_WIDTH - 1) as f64;
                let v = (y as f64 + rn_distr.sample(&mut small_rng)) / (IMAGE_HEIGHT - 1) as f64;
                let ray = camera.get_ray(u, v);
                c += ray_color(&ray, &world, MAX_DEPTH, &mut small_rng);
            }
            img.put_pixel(
                x,
                IMAGE_HEIGHT - 1 - y,
                c.as_rgb_multisample(SAMPLES_PER_PIXEL),
            ); // Image uses inverse y axis direction
        }
        bar.inc(1);
    }
    bar.finish();

    img.save("output.png").unwrap();
}

fn ray_color(ray: &Ray, world: &World, depth: u32, rng: &mut SmallRng) -> Color {
    if depth == 0 {
        return Color::zeros();
    }

    if let Some(hitrecord) = world.hit(ray, 0.001, 1000.) {
        let target = hitrecord.p + hitrecord.normal + random_in_unit_sphere(rng);
        // Color by normal
        return 0.5 * ray_color(&Ray::new(hitrecord.p, target - hitrecord.p), world, depth - 1, rng);
    }
    // Ray hits background
    let unit_dir: Vec3 = ray.direction().normalize(); // .y Range: -1 to 1
    let t = 0.5 * (unit_dir.y + 1.); // Range: 0 to 1
    (1. - t) * Color::new(1., 1., 1.) + t * Color::new(0.5, 0.7, 1.0) // blend
}

fn main() {
    // playground::test_image();
    // playground::test_vectormath();

    raytracer();
}
