use bevy::{math::FloatPow, prelude::*};

const EPSILON_TOLERANCE: f32 = 1e-5; 

pub struct FrameMotion{
	pub direction:Vec3,
	pub distance:f32,
}

#[derive( Debug, Clone)]
pub enum ColliderShape{
	Cylinder(CylinderTarget), 
	Sphere(SphereTarget),
	Plane(PlaneTarget),
}

#[derive(Component, Debug, Clone)]
pub struct Collider{
	pub	shape:ColliderShape,
}

pub struct SphereSweep {
	pub start: Vec3,
	pub movement:FrameMotion,
	pub radius: f32,
}

pub struct HitResult {
	pub time: f32,        // t between 0.0 and 1.0
	pub point: Vec3,       // World position of contact
  pub normal: Vec3,      // World normal pointing away from surface
}

pub trait Collidable {
	fn broad_phase(&self, origin:Vec3, movement:FrameMotion, sphere: &SphereSweep) -> bool;
	fn narrow_phase(&self, origin:Vec3, movement:FrameMotion, sphere: &SphereSweep) -> Option<HitResult>;
}

#[derive( Debug, Clone)]
pub struct CylinderTarget{
	pub direction:Vec3, // from base to tip, normalized
	pub radius:f32,
	pub length:f32,
}

impl Collidable for CylinderTarget{
	fn broad_phase(&self, origin:Vec3, movement:FrameMotion, sphere: &SphereSweep) -> bool {
		//aint hittin' shit if we ain't movin'
		if sphere.movement.distance + movement.distance < EPSILON_TOLERANCE {  return false; }
		//vector of cylinder from base to cap
		let vec_cylinder = self.direction * self.length;
		//vector from base to sphere
		let vec_sphere = sphere.start - origin;
		//distance along cylinder 
		let t = (vec_sphere.dot(vec_cylinder) / self.length.squared()).clamp(0.,1.);
		//world position
		let closest_point = origin + t * vec_cylinder;
		let max_dist = self.radius + sphere.radius + sphere.movement.distance + movement.distance;
		let dist_sq = (sphere.start - closest_point).length_squared();
		dist_sq <= max_dist.squared()
	}

	fn narrow_phase(&self, origin:Vec3, movement:FrameMotion, sphere: &SphereSweep) -> Option<HitResult> {
		
    let to_local_rotation = Quat::from_rotation_arc(self.direction, Vec3::Y);
		let local_start = to_local_rotation * (sphere.start - origin);
		let combined_velocity = sphere.movement.direction * sphere.movement.distance - movement.direction * movement.distance;
		let local_combined_velocity = to_local_rotation * combined_velocity;

		let start_2d = local_start.xz();
		let velocity_2d = local_combined_velocity.xz();
		let total_radius = sphere.radius + self.radius;

		//A*t^2 + B*t + C = 0
		let a = velocity_2d.length_squared();
		let b = 2. * start_2d.dot(velocity_2d);
		let c = start_2d.length_squared() - total_radius.squared();

		let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
      return None; // No mathematical intersection
    }

 		// Solve for the earliest positive time of impact (t)
    let sqrt_disc = discriminant.sqrt();
    let t1 = (-b - sqrt_disc) / (2.0 * a);
    let t2 = (-b + sqrt_disc) / (2.0 * a);

    // We want the smallest t within the frame step [0.0, 1.0]
    let mut t = f32::MAX;
    if (0.0..=1.0).contains(&t1) { t = t1; }
    if (0.0..=1.0).contains(&t2) && t2 < t { t = t2; }

    if t == f32::MAX {
        return None; // Intersection happens outside this frame's movement window
    }

    // Verify the Y-Axis Bounds (Did it hit the actual cylinder length?)
    let hit_y = local_start.y + t * local_combined_velocity.y;
    if hit_y > self.length || hit_y < 0. {
        return None; // Missed above or below the cylinder body
    }

    let local_hit_point = local_start + (t * local_combined_velocity);
    
    // Normal in local space ignores Y because it pushes out radially from the central axis
    let local_normal = Vec3::new(local_hit_point.x, 0.0, local_hit_point.z).normalize_or_zero();

    // 7. Transform back to World Space
    let from_local_rotation = to_local_rotation.inverse();
    let world_normal = from_local_rotation * local_normal;
    let world_hit_point = sphere.start + t * sphere.movement.distance * sphere.movement.direction;

    Some(HitResult {
        time: t,
        point: world_hit_point,
        normal: world_normal,
    })
	}
}

#[derive( Debug, Clone)]
pub struct PlaneTarget{
	pub normal:Vec3,
}

impl Collidable for PlaneTarget{

	fn broad_phase(&self, _origin:Vec3, _movement:FrameMotion, sphere: &SphereSweep) -> bool {
		sphere.movement.distance  > EPSILON_TOLERANCE
	}

	fn narrow_phase(&self, origin:Vec3, _movement:FrameMotion, sphere: &SphereSweep) -> Option<HitResult> {
		let denominator = sphere.movement.direction.dot(self.normal);
		if denominator >= -EPSILON_TOLERANCE { return None; }

		// Shift plane toward the sphere origin by the sphere's radius
		let shifted_plane_origin = origin + self.normal * sphere.radius;
		
		// Distance along the ray to the point of impact
		let hit_distance = (shifted_plane_origin - sphere.start).dot(self.normal) / denominator;

		if hit_distance >= 0.0 && hit_distance <= sphere.movement.distance {
			let t = hit_distance / sphere.movement.distance;
			let world_hit_point = sphere.start + hit_distance * sphere.movement.direction;
			Some(HitResult {
					time: t,
					point: world_hit_point,
					normal: self.normal,
			})
		} else {
			None
		}

	}
}

#[derive( Debug, Clone)]
pub struct SphereTarget{
	pub radius:f32,
}

impl Collidable for SphereTarget{
	fn broad_phase(&self, origin:Vec3, movement:FrameMotion, sphere: &SphereSweep) -> bool {
		if sphere.movement.distance + movement.distance < EPSILON_TOLERANCE {  return false; }
		let max_dist = sphere.radius + self.radius + sphere.movement.distance + movement.distance;
		sphere.start.distance_squared(origin) < max_dist.squared()
	}

	fn narrow_phase(&self, origin:Vec3, movement:FrameMotion,  sphere: &SphereSweep) -> Option<HitResult> {
		
		let vec_sphere = sphere.start - origin;
		let total_radius = sphere.radius + self.radius;
		let velocity = (sphere.movement.direction * sphere.movement.distance) - (movement.direction * movement.distance);

		let a = velocity.length_squared();
		let b = vec_sphere.dot(velocity);
		let c = vec_sphere.length_squared() - total_radius.squared();

		let discriminant = (b * b - 4.0 * a * c);
		if discriminant < 0. { return None;} 
		let discriminant_sqrt = discriminant.sqrt();
		let t1 = (-b - discriminant_sqrt) / (2. * a);
		let t2 = (-b + discriminant_sqrt) / (2. * a);

		let mut t = f32::MAX;
		if (0.0..=1.0).contains(&t1) { t = t1; }
		if (0.0..=1.0).contains(&t2) && t2 < t { t = t2; }

		if t == f32::MAX { return None; }

		let world_hit_point = sphere.start + t * velocity;
		let normal = (world_hit_point - origin).normalize_or_zero();

		Some(HitResult {
			time: t,
			point: world_hit_point,
			normal,
		})

	}
}
