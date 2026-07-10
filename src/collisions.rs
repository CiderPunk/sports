use bevy::prelude::*;


pub struct SphereCast{
	pub origin: Vec3,
	pub direction:Dir3,
	pub radius:f32,
	pub distance:f32,
}

#[derive(Copy, Clone)]
pub struct HitResult{
	pub distance: f32,
	pub position: Vec3,
	pub entity:Entity,
	pub normal:Dir3,
}



impl SphereCast{

	/*
	pub fn intersects_sphere(
		&self,
		target_position:Vec3, 
		target_radius:f32, 
		target_entity:Entity
	)->Option<HitResult>{
		let combined_radius = self.radius + target_radius;
		let p = self.origin - target_position;

		let b = 2. * p.dot(*self.direction);
		let c = p.dot(p) - (combined_radius * combined_radius);
		
		let discriminant = b * b -4.0 * c;
		if discriminant < 0.{
			return None;
		}

		let t = (-b-discriminant.sqrt()) / 2.;
		if t >= 0.0 && t <= self.distance{
			let hit_position = self.origin + *self.direction * t;
			let normal = (hit_position - target_position).normalize();
			
			Some(HitResult { 
				distance: t, 
				position: hit_position, 
				entity: target_entity,
			})
		} else{ None }
	}
 */

	pub fn interset_sphere_vertical_cylinder(
		&self, 
		target_position:Vec3, 
		target_radius:f32, 
		target_height:f32,
		entity:Entity,
	)->Option<HitResult>{
		//https://www.youtube.com/watch?v=ebzlMOw79Yw&t=4s
		let combined_radius = self.radius + target_radius;
		let ray_origin_2d = self.origin.xz();
		let ray_dir_2d = self.direction.xz();
		let ray_len_2d = ray_dir_2d.length();
		if ray_len_2d < f32::EPSILON{
			return None;
		}

		let norm_ray_dir_2d = ray_dir_2d / ray_len_2d;
		let target_position_2d =  target_position.xz();
		let s = ray_origin_2d - target_position_2d;
		let b = s.dot(norm_ray_dir_2d);
		let c = s.dot(s) - (combined_radius * combined_radius);
		let h = b * b - c;
		if h < 0.0{
			return None;
		}

 		let t = -b - h.sqrt();
		if t < 0.0{
			return None;

		}

		let distance = t *(1./ray_len_2d);
		if distance < self.distance{
			let collison_3d = self.direction * t * (1./ray_len_2d);
			let collision_position = collison_3d + self.origin;

			if collision_position.y > target_position.y 
			 	&& collision_position.y < target_position.y + target_height{

				let normal_2d_raw = collision_position.xz() - target_position_2d;
				let normal_2d = normal_2d_raw / combined_radius;

				return Some(HitResult{
					distance,
					position:collision_position,
					entity, 
					normal:Dir3::from_xyz(normal_2d.x, 0., normal_2d.y).unwrap_or(Dir3::X),
				});
			}
		
		}
		None				

	}
}
