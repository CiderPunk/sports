use bevy::prelude::*;

use crate::physics::FrameMotion;



pub struct MotionPlugin;
impl Plugin for MotionPlugin{

	fn build(&self, app: &mut App) {
		app
			.add_systems(Update, (interpolate_translation, interpolate_rotation))
			.add_systems(FixedPreUpdate, (update_child_offset, (store_last_translation, store_last_rotation)).chain())
		;
	}
}


fn update_child_offset(
	mut children:Query<(&PhysicalParent, &ChildPhysicalOffset, &mut PhysicalTranslation, &mut Velocity, )>,
	parent_query:Query<(&PhysicalTranslation, &PhysicalRotation, &Velocity), Without<ChildPhysicalOffset>>
){
	for (parent, offset, mut translation, mut velocity) in children.iter_mut(){
		if let Ok((parent_translation, parent_rotation, parent_velocity)) = parent_query.get(parent.0){
			translation.0 = (parent_rotation.0 * offset.0) + parent_translation.0;
			velocity.direction = parent_velocity.direction;
			velocity.speed = parent_velocity.speed;
		}
	}
}

#[derive(Component)]
#[relationship(relationship_target = PhysicalChildren)]
#[require(ChildPhysicalOffset)]
pub struct PhysicalParent(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = PhysicalParent)]
pub struct PhysicalChildren(Vec<Entity>);


#[derive(Component, Clone, Default)]
#[require(PhysicalTranslation, Velocity)]
pub struct ChildPhysicalOffset(pub Vec3);



#[derive(Component, Clone, Default)]
#[require(PreviousTranslation, Transform)]
pub struct PhysicalTranslation(pub Vec3);


#[derive(Component)]
#[require(PreviousRotation)]
pub struct PhysicalRotation(pub Quat);





#[derive(Component, Default)]
pub struct PreviousTranslation(pub Vec3);

#[derive(Component, Default)]
pub struct PreviousRotation(pub Quat);


#[derive(Component,Clone,Default)]
pub struct Static;




#[derive(Component, Debug, Clone, Copy)]
pub struct Velocity{
	pub direction:Dir3,
	pub speed:f32,
}


impl Default for Velocity{
	fn default() -> Self {
		Self{ direction: Dir3::Y, speed: 0.}
	}
}
impl Velocity{
	pub const ZERO: Self = Self{ direction: Dir3::Y, speed: 0.};

	pub fn zero(&mut self) -> Velocity{
		self.direction= Dir3::Y;
		self.speed = 0.;
		*self
	}
	pub fn to_vec3(&self) -> Vec3{
		self.direction * self.speed
	}

	pub fn from_vec3(value:Vec3) -> Option<Self>{
		let (direction, speed) = Dir3::new_and_length(value).ok()?;
		Some(Self{direction, speed})
	}

	pub fn as_frame_motion(&self, origin: Vec3, offset_seconds:f32, frame_period:f32) -> FrameMotion{
		FrameMotion{
			origin: origin + (self.direction * self.speed * offset_seconds),
			direction: self.direction,
			distance: self.speed * frame_period,
		}
	}
}





fn store_last_translation(
	query:Query<(&mut PreviousTranslation, &PhysicalTranslation)>,
){
	for (mut prev, current) in query{
		prev.0 = current.0;
	}
}

fn interpolate_translation(
  fixed_time: Res<Time<Fixed>>,
	query:Query<(
			&mut Transform,
			&PhysicalTranslation, 
			&PreviousTranslation, 
		), Without<Static>>,
){
	let fraction = fixed_time.overstep_fraction();
	for (mut transform, phys_translation, prev_translation) in query{	
		transform.translation = prev_translation.0.lerp(phys_translation.0, fraction);
	}
}


fn store_last_rotation(
	query:Query<(&mut PreviousRotation, &PhysicalRotation)>,
){
	for (mut prev, current) in query{
		prev.0 = current.0;
	}
}

fn interpolate_rotation(
  fixed_time: Res<Time<Fixed>>,
	query:Query<(
			&mut Transform,
			&PhysicalRotation, 
			&PreviousRotation,
		), Without<Static>>,
){
	let fraction = fixed_time.overstep_fraction();
	for (mut transform, phys_rotation, prev_rotation) in query{	
		transform.rotation =  prev_rotation.0.lerp(phys_rotation.0, fraction);
	}
}
