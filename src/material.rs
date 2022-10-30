use std::sync::Arc;

use crate::hittables::HitRecord;
use crate::util::{Ray, Color, random_unit_vector, near_zero, reflect};
use rand::rngs::SmallRng;

pub trait Material : Send + Sync {
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
}

impl Metal {
	pub fn new(albedo: Color) -> Arc<dyn Material> {
		Arc::new(Metal { albedo })
	}	
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, rec: &HitRecord, _rng: &mut SmallRng) -> Option<(Color, Ray)> {
        let reflected = reflect(&ray.direction().normalize(), &rec.normal);
        if reflected.dot(&rec.normal) > 0. {
	        let scattered = Ray::new(rec.p, reflected);
	        Some((self.albedo, scattered))
        } else {
        	None
        }
    }
}