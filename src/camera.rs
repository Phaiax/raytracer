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
	/// vfov: vertical field of view
    pub fn new(vfov: f64, aspect_ratio: f64) -> Self {
    	let theta = vfov.to_radians();
    	let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = viewport_height * aspect_ratio;

        let focal_length = 1.0;

        let origin = Vec3::new(0., 0., 0.);
        let horizontal = Vec3::new(viewport_width, 0., 0.);
        let vertical = Vec3::new(0., viewport_height, 0.);
        let lower_left_corner =
            origin - horizontal / 2. - vertical / 2. - Vec3::new(0., 0., focal_length);

        Camera {
            origin,
            lower_left_corner,
            vertical,
            horizontal,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray{
        Ray::new(
            self.origin,
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin,
        )
    }
}
