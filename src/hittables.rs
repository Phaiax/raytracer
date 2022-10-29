use std::rc::Rc;

use crate::util::{AsRgb, Color, Point3, Ray, Vec3};

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f32,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(p: Vec3, outward_normal: &Vec3, t: f32, ray: &Ray) -> HitRecord {
        let front_face = ray.direction().dot(outward_normal) < 0.;
        let normal = if front_face {
            *outward_normal
        } else {
            -outward_normal
        };
        HitRecord {
            p,
            normal,
            t,
            front_face,
        }
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(cx: f32, cy: f32, cz: f32, r: f32) -> Rc<dyn Hittable> {
        Rc::new(Sphere {
            center : Point3::new(cx, cy, cz),
            radius: r,
        })
    }
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
