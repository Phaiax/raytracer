use crate::util::{AsRgb, Color, Point3, Ray, Vec3};

pub struct Camera {
    /// Eye
    origin: Point3,
    /// Lower left corner of viewport
    lower_left_corner: Point3,
    /// Vector from lower_left_corner to right side of viewport
    vertical: Vec3,
    /// Vector from lower_left_corner to top side of viewport
    horizontal: Vec3,
}

impl Camera {
    /// vup: Defines `up` for camera
    /// vfov: vertical field of view
    pub fn new(lookfrom: Point3, lookat: Point3, vup: Vec3, vfov: f64, aspect_ratio: f64) -> Self {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (lookfrom - lookat).normalize();
        let u = vup.cross(&w).normalize();
        let v = w.cross(&u);

        let horizontal = viewport_width * u;
        let vertical = viewport_height * v;
        let lower_left_corner = lookfrom - horizontal / 2. - vertical / 2. - w;

        Camera {
            origin: lookfrom,
            lower_left_corner,
            vertical,
            horizontal,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin,
        )
    }
}
