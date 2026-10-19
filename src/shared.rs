use bevy::prelude::*;

use crate::physics::Collider;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PitchAreaMask(pub u32);

// Provide standard bitwise operations for your project
impl PitchAreaMask {
	pub const ANY: Self = Self(1 << 0);
	pub const NORTH: Self = Self(1 << 1);
	pub const SOUTH: Self = Self(1 << 2); 
	pub const INSIDE_GOAL: Self = Self(1 << 3);
	pub const OUTSIDE_GOAL: Self = Self(1 << 4); 

	pub const SOUTH_OUTSIDE_GOAL: Self = Self(PitchAreaMask::SOUTH.0 | PitchAreaMask::OUTSIDE_GOAL.0);
	pub const NORTH_OUTSIDE_GOAL: Self = Self(PitchAreaMask::NORTH.0 | PitchAreaMask::OUTSIDE_GOAL.0);

	pub fn contains(&self, other: Self) -> bool {
		(self.0 & other.0) != 0
	}	

	pub fn combine(&self, other: Self) -> Self {
		Self(self.0 | other.0)
	}
}

impl Default for PitchAreaMask {
	fn default() -> Self {
		Self::ANY
	}
}


pub type SoccerCollider = Collider<PitchAreaMask>;
