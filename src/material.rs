use std::ops::Neg;
use std::sync::Arc;

use crate::hittables::HitRecord;
use crate::util::{
    near_zero, random_in_unit_sphere, random_unit_vector, reflect, refract, Color, Ray,
};
use rand::distributions::Uniform;
use rand::prelude::Distribution;
use rand::rngs::SmallRng;

pub trait Material: Send + Sync {
    /// First return parameter is attenuation
    fn scatter(&self, ray: &Ray, rec: &HitRecord, rng: &mut SmallRng) -> Option<(Color, Ray)>;
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Arc<dyn Material> {
        Arc::new(Lambertian { albedo })
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray: &Ray, rec: &HitRecord, rng: &mut SmallRng) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + random_unit_vector(rng);

        if near_zero(&scatter_direction) {
            scatter_direction = rec.normal;
        }

        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Arc<dyn Material> {
        Arc::new(Metal { albedo, fuzz })
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, rec: &HitRecord, rng: &mut SmallRng) -> Option<(Color, Ray)> {
        let reflected = reflect(&ray.direction().normalize(), &rec.normal);
        if reflected.dot(&rec.normal) > 0. {
            let scattered = Ray::new(rec.p, reflected + self.fuzz * random_in_unit_sphere(rng));
            Some((self.albedo, scattered))
        } else {
            None
        }
    }
}

pub struct Dielectric {
    pub ir: f64,
}

impl Dielectric {
    pub fn new(ir: f64) -> Arc<dyn Material> {
        Arc::new(Dielectric { ir })
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        let r0sq = r0 * r0;
        r0sq + (1.0 - r0) * (1.0 - cosine).powf(5.0)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, rec: &HitRecord, rng: &mut SmallRng) -> Option<(Color, Ray)> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let refaction_ratio = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };
        let unit_direction = ray.direction().normalize();
        let cos_theta = unit_direction.neg().dot(&rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let cannot_refract = refaction_ratio * sin_theta > 1.0;

        let dist: Uniform<f64> = Uniform::new(0.0, 1.0);
        let direction = if cannot_refract
            || Dielectric::reflectance(cos_theta, refaction_ratio) > dist.sample(rng)
        {
            reflect(&unit_direction, &rec.normal)
        } else {
            refract(&unit_direction, &rec.normal, refaction_ratio)
        };
        Some((attenuation, Ray::new(rec.p, direction)))
    }
}
