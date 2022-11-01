use std::sync::Arc;

use nalgebra::Matrix3;

use crate::material::Material;
use crate::util::{AsRgb, Color, Point3, Ray, Vec3};

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub material: Arc<dyn Material>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(
        p: Vec3,
        outward_normal: &Vec3,
        material: &Arc<dyn Material>,
        t: f64,
        ray: &Ray,
    ) -> HitRecord {
        let front_face = ray.direction().dot(outward_normal) < 0.;
        let normal = if front_face {
            *outward_normal
        } else {
            -outward_normal
        };
        HitRecord {
            p,
            normal,
            material: material.clone(),
            t,
            front_face,
        }
    }
}

pub trait Hittable: Sync + Send {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(
        cx: f64,
        cy: f64,
        cz: f64,
        r: f64,
        material: &Arc<dyn Material>,
    ) -> Arc<dyn Hittable> {
        Arc::new(Sphere {
            center: Point3::new(cx, cy, cz),
            radius: r,
            material: material.clone(),
        })
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc: Vec3 = r.origin() - self.center;
        let a: f64 = r.direction().magnitude_squared();
        let half_b: f64 = oc.dot(&r.direction());
        let c: f64 = oc.magnitude_squared() - self.radius * self.radius;
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
            Some(HitRecord::new(p, &normal, &self.material, t, &r))
        }
    }
}

pub struct Cylinder {
    pub start: Point3,
    pub dir: Vec3,
    pub radius: f64,
    pub material: Arc<dyn Material>,
}

impl Cylinder {
    pub fn new(
        start: Point3,
        dir: Vec3,
        radius: f64,
        material: &Arc<dyn Material>,
    ) -> Arc<dyn Hittable> {
        Arc::new(Cylinder {
            start,
            dir,
            radius,
            material: material.clone(),
        })
    }
}

impl Hittable for Cylinder {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let (t_ray, n, p_ray, p_centerline) = nearest_points(r.orig, r.dir, self.start, self.dir);
        let d = (p_ray - p_centerline).magnitude();
        if d < self.radius && t_min < t_ray && t_ray < t_max {
            // println!("{} !!", d);
            Some(HitRecord::new(p_ray, &n, &self.material, t_ray, &r))
        } else {
            // println!("{}", d);
            None
        }
    }
}

#[allow(non_snake_case)]
pub fn nearest_points(K: Point3, l: Vec3, A: Point3, b: Vec3) -> (f64, Vec3, Point3, Point3) {
    let n = l.cross(&b);
    let m = Matrix3::from_columns(&[l, n, b]);
    let t = m.try_inverse().unwrap() * (A - K);
    let p1 = K + t.x * l;
    let p2 = p1 + t.y * n;

    (t.x, n, p1, p2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use approx::assert_relative_ne;

    #[rustfmt::skip]
    #[test]
    fn test_nearest_points() {
        let (t, _n, p1, p2) = nearest_points(Point3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0), 
                                            Point3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(p1, Point3::new(0.0, 0.0, 1.0));
        assert_eq!(p2, Point3::new(0.0, 1.0, 1.0));
        assert_eq!(t, 0.0);

        // Test: With angled lines
        let l = Vec3::new(1.0, 0.2, 0.0);
        let b = Vec3::new(0.2, 0.0, 1.0);
        let (t, _n, p1, p2) = nearest_points(Point3::new(0.0, 0.0, 1.0), l, 
                                            Point3::new(0.0, 1.0, 0.0), b);
        // Values taken from test, but they are logically consistent
        assert_relative_eq!(p1, Point3::new(0.3917050, 0.0783410, 1.0), epsilon = 1e-5);
        assert_relative_eq!(p2, Point3::new(0.2073732, 1.0, 1.0368663), epsilon = 1e-5);
        assert_relative_eq!(t, 0.3917050, epsilon = 1e-5);

        // Test: Independence of line origin and length of direction vector gives same results
        let (t_2, _n_2, p1_2, p2_2) = nearest_points(Point3::new(0.0, 0.0, 1.0) + 200.0 * l, 10.0 * l, 
                                                    Point3::new(0.0, 1.0, 0.0) - 14.2*b, 0.1 * b);
        assert_relative_ne!(t, t_2, epsilon = 1e-8);
        assert_relative_eq!(p1, p1_2, epsilon = 1e-8);
        assert_relative_eq!(p2, p2_2, epsilon = 1e-8);


    }
}
