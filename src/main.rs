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
mod gui;
mod hittables;
mod material;
mod playground;
mod util;
mod world;

use std::error::Error;
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};

use crate::camera::Camera;
use crate::hittables::{Hittable, Sphere};
use crate::util::{random_unit_vector, AsRgb, Color, Point3, Ray, Vec3};
use crate::world::World;
use camera::CameraBuilder;
use clap::Parser;
use eframe::epaint::{Color32, ColorImage};
use hittables::Cylinder;
use image::{ImageBuffer, Rgba, RgbaImage};
use indicatif::ProgressBar;
use material::{Dielectric, Lambertian, Metal};
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use util::{vec3_random, ProgressBarWrapper};

#[derive(Parser, Debug)]
#[command(version)]
pub struct Args {
    #[command(flatten)]
    raytrace_params: RaytraceParams,
    #[arg(short, long, default_value = "output.png")]
    output_filename: String,
    #[arg(short, long, default_value_t = false)]
    gui: bool,
}

#[derive(Parser, Debug, Clone)]
#[command()]
pub struct RaytraceParams {
    #[arg(short, long = "width", default_value_t = 400)]
    pub image_width: u32,
    #[arg(short, long, default_value = "16:9", value_parser = parse_aspect_ratio)]
    pub aspect_ratio: f64,
    #[arg(short, long, default_value_t = 10)]
    pub samples_per_pixel: u32,
    #[arg(short, long, default_value_t = 50)]
    pub max_depth: u32,
}

type F64RgbaImage = ImageBuffer<Rgba<f64>, Vec<f64>>;
struct SamplesAdder {
    sum_img: F64RgbaImage,
    num_samples: u32,
}

impl SamplesAdder {
    fn new(width: u32, height: u32) -> Self {
        SamplesAdder {
            sum_img: ImageBuffer::new(width, height),
            num_samples: 0,
        }
    }

    fn add_image(&mut self, step_img: &F64RgbaImage) {
        let step_samples: &[f64] = step_img.as_flat_samples().samples;
        let sum_samples: &mut [f64] = self.sum_img.as_flat_samples_mut().samples;
        for (step_sample, sum_sample) in step_samples.iter().zip(sum_samples.iter_mut()) {
            *sum_sample += *step_sample;
        }
        self.num_samples += 1;
    }

    fn normalized(&self) -> RgbaImage {
        let num_samples = self.num_samples as f64;
        let mut img = RgbaImage::new(self.sum_img.width(), self.sum_img.height());
        let sum_samples = self.sum_img.as_flat_samples().samples;
        let img_samples = img.as_flat_samples_mut().samples;
        for (sum_sample, img_sample) in sum_samples.iter().zip(img_samples.iter_mut()) {
            *img_sample = ((*sum_sample / num_samples).sqrt().clamp(0.0, 0.999) * 256.0) as u8;
        }
        img
    }

    fn normalized_colorimage(&self) -> ColorImage {
        let num_samples = self.num_samples as f64;
        let sum_samples = self.sum_img.as_flat_samples().samples;
        let size = [
            self.sum_img.width() as usize,
            self.sum_img.height() as usize,
        ];
        let mut img_pixels: Vec<Color32> = vec![Color32::from_gray(0); size[0] * size[1]];

        for (sum_pixels, img_pixel) in sum_samples.chunks_exact(4).zip(img_pixels.iter_mut()) {
            *img_pixel = Color32::from_rgba_unmultiplied(
                ((sum_pixels[0] / num_samples).sqrt().clamp(0.0, 0.999) * 256.0) as u8,
                ((sum_pixels[1] / num_samples).sqrt().clamp(0.0, 0.999) * 256.0) as u8,
                ((sum_pixels[2] / num_samples).sqrt().clamp(0.0, 0.999) * 256.0) as u8,
                255,
            )
        }
        ColorImage {
            size,
            pixels: img_pixels,
        }
    }
}

pub fn render_live(
    params: &RaytraceParams,
    world: &World,
    camera: &Camera,
    progress: &Box<dyn ProgressBarWrapper>,
    stop: Arc<AtomicBool>,
) -> RgbaImage {
    progress.set_length(params.samples_per_pixel as u64);

    let image_height: u32 = (params.image_width as f64 / params.aspect_ratio) as u32;
    let img: Mutex<SamplesAdder> = Mutex::new(SamplesAdder::new(params.image_width, image_height));

    (0..params.samples_per_pixel).into_par_iter().for_each(|s| {
        if stop.load(Relaxed) {
            return;
        }

        let mut small_rng = SmallRng::seed_from_u64(232008239771 + s as u64);
        let step_img = render_sample(params, world, camera, &mut small_rng, Arc::clone(&stop));

        if stop.load(Relaxed) {
            return;
        }

        img.lock().unwrap().add_image(&step_img);

        if stop.load(Relaxed) {
            return;
        }

        progress.inc(1, &Box::new(|| img.lock().unwrap().normalized_colorimage()));
    });
    progress.finish();
    img.into_inner().unwrap().normalized()
}

pub fn render(
    params: &RaytraceParams,
    world: &World,
    camera: &Camera,
    progress: &Box<dyn ProgressBarWrapper>,
) -> RgbaImage {
    render_live(
        params,
        world,
        camera,
        progress,
        Arc::new(AtomicBool::new(false)),
    )
}

pub fn render_sample(
    params: &RaytraceParams,
    world: &World,
    camera: &Camera,
    rng: &mut SmallRng,
    stop: Arc<AtomicBool>,
) -> F64RgbaImage {
    let image_height: u32 = (params.image_width as f64 / params.aspect_ratio) as u32;
    let mut img: F64RgbaImage = ImageBuffer::new(params.image_width, image_height);
    let rn_distr: Uniform<f64> = Uniform::new(0.0, 1.0);

    for y in 0..image_height {
        for x in 0..params.image_width {
            let u = (x as f64 + rn_distr.sample(rng)) / (params.image_width - 1) as f64;
            let v = (y as f64 + rn_distr.sample(rng)) / (image_height - 1) as f64;
            let ray = camera.get_ray(u, v, rng);
            let c = ray_color(&ray, &world, params.max_depth, rng);
            img.put_pixel(x, image_height - 1 - y, c.as_f64_rgba()); // ImageBuffer uses inverse y axis direction
        }
        if stop.load(Relaxed) {
            break;
        }
    }

    img
}

fn scene_chapter13() -> (World, CameraBuilder) {
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
                    world.add(Sphere::new(
                        center.x,
                        center.y,
                        center.z,
                        0.2,
                        &sphere_material,
                    ));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo: Color = vec3_random(&distr_0p5_1, &mut small_rng);
                    let fuzz = distr_0_1.sample(&mut small_rng) / 2.0;
                    let sphere_material = Metal::new(albedo, fuzz);
                    world.add(Sphere::new(
                        center.x,
                        center.y,
                        center.z,
                        0.2,
                        &sphere_material,
                    ));
                } else {
                    // glass
                    let sphere_material = Dielectric::new(1.5);
                    world.add(Sphere::new(
                        center.x,
                        center.y,
                        center.z,
                        0.2,
                        &sphere_material,
                    ));
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

    let mut camera = CameraBuilder::new();
    camera
        .lookfrom(Point3::new(13.0, 2.0, 3.0))
        .lookat(Point3::new(0.0, 0.0, 0.0))
        .vup(Vec3::new(0.0, 1.0, 0.0))
        .vfov(20.0)
        .aperture(0.1)
        .focus_dist(10.0);

    (world, camera)
}

fn scene_tutorial() -> (World, CameraBuilder) {
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

    let mut camera = CameraBuilder::new();
    camera
        .lookfrom(Point3::new(0.0, 0.0, 1.0))
        .lookat(Point3::new(0.0, 0.0, -1.0))
        .vup(Vec3::new(0.0, 1.0, 0.0))
        .vfov(90.0)
        .aperture(0.0)
        .focus_dist(10.0);

    (world, camera)
}

#[allow(unused_variables)]
fn scene_cylinder() -> (World, CameraBuilder) {
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

    world.add(Cylinder::new(
        Point3::new(0.0, 0.2, -1.0),
        Vec3::new(0.0, 1.0, 0.0),
        0.1,
        &material_red,
    ));

    let mut camera = CameraBuilder::new();
    camera
        .lookfrom(Point3::new(0.0, 0.0, 1.0))
        .lookat(Point3::new(0.0, 0.0, -1.0))
        .vup(Vec3::new(0.0, 1.0, 0.0))
        .vfov(90.0)
        .aperture(0.0)
        .focus_dist(10.0);

    (world, camera)
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

fn parse_aspect_ratio<'a>(
    aspect_ratio: &'a str,
) -> Result<f64, Box<dyn Error + Send + Sync + 'static>> {
    let err = "Aspect ratio format is: '<w>:<h>', e.g.: '16:9'";
    let mut aspect_ratio = aspect_ratio.split(":");
    let w: f64 = aspect_ratio.next().ok_or(err)?.parse().map_err(|_| err)?;
    let h: f64 = aspect_ratio.next().ok_or(err)?.parse().map_err(|_| err)?;
    Ok(w / h)
}

fn main() {
    // playground::test_image();
    // playground::test_vectormath();
    // playground::test_ray_cylinder_math();

    let args = Args::parse();

    // World and Camera
    let (world, mut camera_builder) = scene_cylinder();
    camera_builder.aspect_ratio(args.raytrace_params.aspect_ratio);

    if args.gui {
        crate::gui::run_gui(args.raytrace_params, world, camera_builder);
    } else {
        let progress: Box<dyn ProgressBarWrapper> = Box::new(ProgressBar::new(1));
        let img = render(
            &args.raytrace_params,
            &world,
            &camera_builder.build().unwrap(),
            &progress,
        );
        img.save(args.output_filename)
            .expect("Could not save file.");
    }
}
