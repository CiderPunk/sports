use bevy::prelude::*;
use bevy_enhanced_input::condition::DEFAULT_ACTUATION;
use crate::physics::ColliderShape;

pub const DEFAULT_RESTITUTION:f32 = 0.8;

#[derive(Component, Copy, Clone, Default)]
#[require(ColliderShape, ColliderType, Restitution, PitchCollisionZone)]
pub struct SoccerCollider;



#[derive(Component, Copy, Clone)]
pub struct Restitution(pub f32);

impl Default for Restitution{

	fn default() -> Self{
		Self( DEFAULT_RESTITUTION )
	}
}

#[derive(Component, Clone, Copy, Default, PartialEq, Eq)]
pub enum ColliderType{
	#[default]
	None,
	Tackle(bool),
	Sensor,
	Player,
}



#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PitchCollisionZone(pub u32);

// Provide standard bitwise operations for your project
impl PitchCollisionZone {
	pub const ANY: Self = Self(1 << 0);
	pub const NORTH: Self = Self(1 << 1); //2
	pub const SOUTH: Self = Self(1 << 2); //4
	pub const INSIDE_GOAL: Self = Self(1 << 3); //8
	pub const OUTSIDE_GOAL: Self = Self(1 << 4); //16

	pub const SOUTH_OUTSIDE_GOAL: Self = Self(PitchCollisionZone::SOUTH.0 | PitchCollisionZone::OUTSIDE_GOAL.0);
	pub const NORTH_OUTSIDE_GOAL: Self = Self(PitchCollisionZone::NORTH.0 | PitchCollisionZone::OUTSIDE_GOAL.0);

	pub fn contains(&self, other: Self) -> bool {
		(self.0 & other.0) == other.0
	}	

	pub fn combine(&self, other: Self) -> Self {
		Self(self.0 | other.0)
	}
}

impl Default for PitchCollisionZone {
	fn default() -> Self {
		Self::ANY
	}
}


#[derive(Resource, Clone)]
pub struct PitchConfiguration{
	pub half_width:f32,
	pub half_length:f32,
	pub border:f32,
	pub stripe_count:u32,
	pub line_width:f32,
	pub centre_circle_radius:f32,
	pub penalty_width:f32,
	pub penalty_length:f32,
	pub goal_area_width:f32,
	pub goal_area_length:f32,
	pub half_goal_width:f32,
	pub goal_height:f32,
	pub goal_depth:f32,
	pub corner_arc_radius:f32,
	pub penalty_spot_from_goal:f32,
	pub penalty_arc_radius:f32,
	pub pole_radius:f32,
}

impl PitchConfiguration {
	
	pub fn get_pitch_collision_zone(&self, position:Vec3)->PitchCollisionZone{
		let mut filter = PitchCollisionZone::ANY; 
		if position.z.abs() > self.half_length *0.75{
			filter = filter.combine(
				if position.z.signum() == 1.0 {
					PitchCollisionZone::SOUTH 
				} else { 
					PitchCollisionZone::NORTH
				});
			//out of play?
			if position.z.abs() > self.half_length{
				//in goal???
				filter = filter.combine(
					if position.x.abs() < self.half_goal_width 
					&& position.y < self.goal_height 
					&& position.z.abs() < self.half_length + self.goal_depth{
						PitchCollisionZone::INSIDE_GOAL
					}
					else{
						PitchCollisionZone::OUTSIDE_GOAL
					}
				);
			}
		}
		filter
	}
}


impl Default for PitchConfiguration{
	fn default() -> Self {
		Self { 
			half_width: 32., 
			half_length: 45.,
			stripe_count:16,
			border:5.,
			centre_circle_radius:9.16,
    	penalty_width: 40.32,
			penalty_length:16.5,
			goal_area_width: 18.32,
			goal_area_length: 5.5,
			half_goal_width: 7.32 / 2.,
			goal_height:2.44,
			goal_depth:2.0,
			corner_arc_radius: 1.,
			penalty_spot_from_goal: 11.,
			penalty_arc_radius: 9.15,
    	line_width: 0.2,
			pole_radius:0.1,
		}
	}
}

