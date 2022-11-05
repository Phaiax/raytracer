use rand::rngs::SmallRng;

use crate::util::{random_in_unit_disk, AsRgb, Color, Point3, Ray, Vec3};

#[derive(Clone)]
pub struct CameraBuilder {
    pub lookfrom: Option<Point3>,
    pub lookat: Option<Point3>,
    pub vup: Option<Vec3>,
    pub vfov: Option<f64>,
    pub aspect_ratio: Option<f64>,
    pub aperture: Option<f64>,
    pub focus_dist: Option<f64>,
}

impl CameraBuilder {
    pub fn new() -> Self {
        CameraBuilder {
            lookfrom: None,
            lookat: None,
            vup: None,
            vfov: None,
            aspect_ratio: None,
            aperture: None,
            focus_dist: None,
        }
    }
    pub fn lookfrom(&mut self, lookfrom: Point3) -> &mut Self {
        self.lookfrom = Some(lookfrom);
        self
    }
    pub fn lookat(&mut self, lookat: Point3) -> &mut Self {
        self.lookat = Some(lookat);
        self
    }
    pub fn vup(&mut self, vup: Vec3) -> &mut Self {
        self.vup = Some(vup);
        self
    }
    pub fn vfov(&mut self, vfov: f64) -> &mut Self {
        self.vfov = Some(vfov);
        self
    }
    pub fn aspect_ratio(&mut self, aspect_ratio: f64) -> &mut Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }
    pub fn aperture(&mut self, aperture: f64) -> &mut Self {
        self.aperture = Some(aperture);
        self
    }
    pub fn focus_dist(&mut self, focus_dist: f64) -> &mut Self {
        self.focus_dist = Some(focus_dist);
        self
    }
    pub fn build(&self) -> Option<Camera> {
        Some(Camera::new(
            self.lookfrom?,
            self.lookat?,
            self.vup?,
            self.vfov?,
            self.aspect_ratio?,
            self.aperture?,
            self.focus_dist?,
        ))
    }
}

#[derive(Clone)]
pub struct Camera {
    /// Eye
    origin: Point3,
    /// Lower left corner of viewport
    lower_left_corner: Point3,
    /// Vector from lower_left_corner to right side of viewport
    pub vertical: Vec3,
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
