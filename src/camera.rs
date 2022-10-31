use rand::rngs::SmallRng;

use crate::util::{random_in_unit_disk, AsRgb, Color, Point3, Ray, Vec3};

pub struct Camera {
    /// Eye
    origin: Point3,
    /// Lower left corner of viewport
    lower_left_corner: Point3,
    /// Vector from lower_left_corner to right side of viewport
    vertical: Vec3,
    /// Vector from lower_left_corner to top side of viewport
    horizontal: Vec3,
    ///
    u: Vec3,
    v: Vec3,
    w: Vec3,
    ///
    lens_radius: f64,
}

impl Camera {
    /// vup: Defines `up` for camera
    /// vfov: vertical field of view
    pub fn new(
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Self {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (lookfrom - lookat).normalize();
        let u = vup.cross(&w).normalize();
        let v = w.cross(&u);

        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let lower_left_corner = lookfrom - horizontal / 2. - vertical / 2. - focus_dist * w;

        Camera {
            origin: lookfrom,
            lower_left_corner,
            vertical,
            horizontal,
            u,
            v,
            w,
            lens_radius: aperture / 2.,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64, rng: &mut SmallRng) -> Ray {
        let rd = self.lens_radius * random_in_unit_disk(rng);
        let offset = self.u * rd.x + self.v * rd.y;
        Ray::new(
            self.origin + offset,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset,
        )
    }
}
