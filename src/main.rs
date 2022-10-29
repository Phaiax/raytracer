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

mod hittables;
mod playground;
mod world;
mod util;

use std::rc::Rc;

use crate::hittables::{Hittable, Sphere};
use crate::util::{AsRgb, Color, Point3, Ray, Vec3};
use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;
use world::World;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const IMAGE_WIDTH: u32 = 400;
const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32;

pub fn raytracer() {
    let mut img: RgbImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    let bar = ProgressBar::new(IMAGE_HEIGHT as u64);

    // World

    let mut world = World::new();
    world.add(Sphere::new(0., 0., -1., 0.5));
    world.add(Sphere::new(0., -100.5, -1., 100.));

    // Camera

    let viewport_height = 2.0;
    let viewport_width = viewport_height * ASPECT_RATIO;
    let focal_length = 1.0;

    let origin = Vec3::new(0., 0., 0.);
    let horizontal = Vec3::new(viewport_width, 0., 0.);
    let vertical = Vec3::new(0., viewport_height, 0.);
    let lower_left_corner =
        origin - horizontal / 2. - vertical / 2. - Vec3::new(0., 0., focal_length);

    for y in 0..img.height() {
        for x in 0..img.width() {
            let u = x as f32 / (IMAGE_WIDTH - 1) as f32;
            let v = y as f32 / (IMAGE_HEIGHT - 1) as f32;
            let ray = Ray::new(
                origin,
                lower_left_corner + u * horizontal + v * vertical - origin,
            );
            let c = ray_color(&ray, &world);
            img.put_pixel(x, IMAGE_HEIGHT - 1 - y, c.as_rgb()); // Image uses inverse y axis direction
        }
        bar.inc(1);
    }
    bar.finish();

    img.save("output.png").unwrap();
}

fn ray_color(ray: &Ray, world: &World) -> Color {
    
    if let Some(hitrecord) = world.hit(ray, 0., 1000.) {
        // Color by normal
        return 0.5
            * Color::new(
                hitrecord.normal.x + 1.0,
                hitrecord.normal.y + 1.0,
                hitrecord.normal.z + 1.0,
            );
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
