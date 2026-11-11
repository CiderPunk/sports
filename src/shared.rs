use bevy::prelude::*;
use crate::{physics::Collider, pitch::PitchCollisionZone};

pub type SoccerCollider = Collider<PitchCollisionZone>;


#[derive(Component, Clone, Copy, Default)]
pub enum ColliderType{
	#[default]
	None,
	Tackle(bool),
	Sensor,
}