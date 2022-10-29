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

mod playground;
mod util;

use crate::util::{AsRgb, Color, Point3, Ray, Vec3};
use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const IMAGE_WIDTH: u32 = 400;
const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32;

pub fn raytracer() {
    let mut img: RgbImage = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    let bar = ProgressBar::new(IMAGE_HEIGHT as u64);

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
            let c = ray_color(&ray);
            img.put_pixel(x, IMAGE_HEIGHT - 1 - y, c.as_rgb()); // Image uses inverse y axis direction
        }
        bar.inc(1);
    }
    bar.finish();

    img.save("output.png").unwrap();
}

struct HitRecord {
    p: Point3,
    normal: Vec3,
    t: f32,
    front_face: bool,
}

impl HitRecord {
    fn new(p: Vec3, outward_normal: &Vec3, t: f32, ray: &Ray) -> HitRecord {
        let front_face = ray.direction().dot(outward_normal) > 0.;
        let normal = if front_face { *outward_normal } else { -outward_normal };
        HitRecord {
            p,
            normal,
            t,
            front_face,
        }
    }
}

trait Hittable {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

struct Sphere {
    center: Point3,
    radius: f32,
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc: Vec3 = r.origin() - self.center;
        let a: f32 = r.direction().magnitude_squared();
        let half_b: f32 = oc.dot(&r.direction());
        let c: f32 = oc.magnitude_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            None
        } else {
            let sqrtd = discriminant.sqrt();

            // Find nearest root that lies in acceptable range
            let mut t = (-half_b - sqrtd) / a;
            if t < t_min || t_max < t {
                t = (-half_b + sqrtd) / a;
                if t < t_min || t_max < t {
                    return None;
                }
            }

            let p = r.at(t);
            let normal = (p - self.center) / self.radius;
            Some(HitRecord::new(p, &normal, t, &r))
        }
    }
}


fn ray_color(ray: &Ray) -> Color {
    let sphere0 = Sphere {
        center: Point3::new(0., 0., -1.),
        radius: 0.5
    };
    if let Some(hitrecord) = sphere0.hit(ray, 0., 1000.) {
        return 0.5 * Color::new(hitrecord.normal.x + 1.0, hitrecord.normal.y + 1.0, hitrecord.normal.z + 1.0);    
    }
    // Ray hits background
    let unit_dir: Vec3 = ray.dir.normalize(); // .y Range: -1 to 1
    let t = 0.5 * (unit_dir.y + 1.); // Range: 0 to 1
    (1. - t) * Color::new(1., 1., 1.) + t * Color::new(0.5, 0.7, 1.0) // blend
}

fn main() {
    // playground::test_image();
    // playground::test_vectormath();

    raytracer();
}
