use bevy::prelude::*;

use crate::{physics::Collider, pitch::PitchCollisionZone};



pub type SoccerCollider = Collider<PitchCollisionZone>;



#[derive(Component)]
pub enum ColliderType{
	Tackle,
	Sensor,
}