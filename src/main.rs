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
mod material;
mod playground;
mod util;
mod world;

use std::rc::Rc;
use std::sync::Mutex;

use crate::camera::Camera;
use crate::hittables::{Hittable, Sphere};
use crate::util::{random_unit_vector, AsRgb, Color, Point3, Ray, Vec3};
use crate::world::World;
use hittables::Cylinder;
use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;
use material::{Dielectric, Lambertian, Metal};
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use util::vec3_random;

const ASPECT_RATIO: f64 = 3.0 / 2.0;
const IMAGE_WIDTH: u32 = 400;
const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as u32;
const SAMPLES_PER_PIXEL: u32 = 10;
const MAX_DEPTH: u32 = 50;

pub fn raytracer() {
    let img: Mutex<RgbImage> = Mutex::new(ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT));
    let bar = ProgressBar::new(IMAGE_HEIGHT as u64);

    // World
    let world = scene_cylinder();

    // Camera
    let lookfrom = Point3::new(0.0, 0.0, 1.0);
    let lookat = Point3::new(0.0, 0.0, -1.0);
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::new(0.0, 1.0, 0.0),
        90.0,
        ASPECT_RATIO,
        0.0, // aperture
        10.0, // dist_to_focus
    );

    let range = 0..IMAGE_HEIGHT;
    range.into_par_iter().for_each(|y| {
        let mut small_rng = SmallRng::seed_from_u64(232008239771 + y as u64);
        let rn_distr: Uniform<f64> = Uniform::new(0.0, 1.0);
        for x in 0..IMAGE_WIDTH {
            let mut c = Color::zeros();
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (x as f64 + rn_distr.sample(&mut small_rng)) / (IMAGE_WIDTH - 1) as f64;
                let v = (y as f64 + rn_distr.sample(&mut small_rng)) / (IMAGE_HEIGHT - 1) as f64;
                let ray = camera.get_ray(u, v, &mut small_rng);
                c += ray_color(&ray, &world, MAX_DEPTH, &mut small_rng);
            }
            img.lock().unwrap().put_pixel(
                x,
                IMAGE_HEIGHT - 1 - y,
                c.as_rgb_multisample(SAMPLES_PER_PIXEL),
            ); // Image uses inverse y axis direction
        }
        bar.inc(1);
    });
    bar.finish();

    img.lock().unwrap().save("output.png").unwrap();
}

fn scene_chapter13() -> World {
    let mut world = World::new();
    let mut small_rng = SmallRng::seed_from_u64(23428359242 as u64);
    let distr_0_1: Uniform<f64> = Uniform::new(0.0, 1.0);
    let distr_0p5_1: Uniform<f64> = Uniform::new(0.5, 1.0);

    let material_ground = Lambertian::new(Color::new(0.5, 0.5, 0.5));
    world.add(Sphere::new(0.0, -1000.0, 0.0, 1000.0, &material_ground));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = distr_0_1.sample(&mut small_rng);
            let center = Point3::new(
                a as f64 + 0.9 * distr_0_1.sample(&mut small_rng),
                0.2,
                b as f64 + 0.9 * distr_0_1.sample(&mut small_rng),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).magnitude() > 0.9 {
                if choose_mat < 0.8 {
                    // diffse
                    let albedo: Color = vec3_random(&distr_0_1, &mut small_rng)
                        .component_mul(&vec3_random(&distr_0_1, &mut small_rng));
                    let sphere_material = Lambertian::new(albedo);
                    world.add(Sphere::new(center.x, center.y, center.z, 0.2, &sphere_material));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo: Color = vec3_random(&distr_0p5_1, &mut small_rng);
                    let fuzz = distr_0_1.sample(&mut small_rng) / 2.0;
                    let sphere_material = Metal::new(albedo, fuzz);
                    world.add(Sphere::new(center.x, center.y, center.z, 0.2, &sphere_material));
                } else {
                    // glass
                    let sphere_material = Dielectric::new(1.5);
                    world.add(Sphere::new(center.x, center.y, center.z, 0.2, &sphere_material));
                }
            }
        }
    }

    let material1 = Dielectric::new(1.5);
    world.add(Sphere::new(0.0, 1.0, 0.0, 1.0, &material1));

    let material2 = Lambertian::new(Color::new(0.4, 0.2, 0.1));
    world.add(Sphere::new(-4.0, 1.0, 0.0, 1.0, &material2));

    let material3 = Metal::new(Color::new(0.7, 0.6, 0.5), 0.0);
    world.add(Sphere::new(4.0, 1.0, 0.0, 1.0, &material3));

    world
}

fn scene_tutorial() -> World {
    let material_ground = Lambertian::new(Color::new(0.8, 0.8, 0.0));
    let material_center = Lambertian::new(Color::new(0.1, 0.2, 0.5));
    let material_left = Dielectric::new(1.5);
    let material_right = Metal::new(Color::new(0.8, 0.6, 0.2), 0.0);

    let mut world = World::new();
    world.add(Sphere::new(0.0, -100.5, -1.0, 100.0, &material_ground));
    world.add(Sphere::new(0.0, 0.0, -1.0, 0.5, &material_center));
    world.add(Sphere::new(-1.0, 0.0, -1.0, 0.5, &material_left));
    world.add(Sphere::new(-1.0, 0.0, -1.0, -0.45, &material_left));
    world.add(Sphere::new(1.0, 0.0, -1.0, 0.5, &material_right));
    world
}

#[allow(unused_variables)]
fn scene_cylinder() -> World {
    let material_ground = Lambertian::new(Color::new(0.8, 0.8, 0.0));
    let material_center = Lambertian::new(Color::new(0.1, 0.2, 0.5));
    let material_left = Dielectric::new(1.5);
    let material_right = Metal::new(Color::new(0.8, 0.6, 0.2), 0.0);
    let material_red = Lambertian::new(Color::new(1.0, 0.0, 0.0));

    let mut world = World::new();
    world.add(Sphere::new(0.0, -100.5, -1.0, 100.0, &material_ground));
    // world.add(Sphere::new(0.0, 0.0, -1.0, 0.5, &material_center));
    world.add(Sphere::new(-1.0, 0.0, -1.0, 0.5, &material_left));
    world.add(Sphere::new(-1.0, 0.0, -1.0, -0.45, &material_left));
    world.add(Sphere::new(1.0, 0.0, -1.0, 0.5, &material_right));

    world.add(Cylinder::new(Point3::new(0.0, 0.2, -1.0), Vec3::new(0.0, 1.0, 0.0), 0.1, &material_red));
    world
}

fn ray_color(ray: &Ray, world: &World, depth: u32, rng: &mut SmallRng) -> Color {
    if depth == 0 {
        return Color::zeros();
    }

    if let Some(hitrecord) = world.hit(ray, 0.001, 1000.) {
        if let Some((attenuation, scatterray)) = hitrecord.material.scatter(ray, &hitrecord, rng) {
            return attenuation.component_mul(&ray_color(&scatterray, world, depth - 1, rng));
        } else {
            return Color::zeros();
        }
    }
    // Ray hits background
    let unit_dir: Vec3 = ray.direction().normalize(); // .y Range: -1 to 1
    let t = 0.5 * (unit_dir.y + 1.); // Range: 0 to 1
    (1. - t) * Color::new(1., 1., 1.) + t * Color::new(0.5, 0.7, 1.0) // blend
}

fn main() {
    // playground::test_image();
    // playground::test_vectormath();
    // playground::test_ray_cylinder_math();
    raytracer();
}
