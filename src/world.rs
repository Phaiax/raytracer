
use std::{sync::Arc};

use crate::{hittables::{Hittable, HitRecord}, util::Ray};

pub struct World {
	objects: Vec<Arc<dyn Hittable>>,
}

impl World {
	pub fn new() -> Self {
		World { objects: vec![] }
	}

	pub fn add(&mut self, hittable: Arc<dyn Hittable>) {
		self.objects.push(hittable)
	}

	pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
		let mut hit_record = None;
		let mut closest_so_far = t_max;

		for object in self.objects.iter() {
			if let Some(new_hit_record) = object.hit(r, t_min, closest_so_far) {
				closest_so_far = new_hit_record.t;
				hit_record = Some(new_hit_record);
			}
		}

		return hit_record;
	}
}