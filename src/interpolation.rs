use bevy::prelude::*;

pub struct InterpolationPlugin;

impl Plugin for InterpolationPlugin{
	fn build(&self, app: &mut App) {

		app
			.add_systems(Update, interpolate_transform)
			.add_systems(FixedPreUpdate, (store_last_translation, store_last_rotation))
		;
	}
}

#[derive(Component)]
#[require(PreviousTranslation)]
pub struct PhysicalTranslation(pub Vec3);


#[derive(Component, Default)]
pub struct PreviousTranslation(pub Vec3);

#[derive(Component)]
#[require(PreviousRotation)]
pub struct PhysicalRotation(pub Quat);

#[derive(Component, Default)]
pub struct PreviousRotation(pub Quat);


#[derive(Component)]
pub struct Static;


fn store_last_rotation(
	query:Query<(&mut PreviousRotation, &PhysicalRotation)>,
){
	for (mut prev, current) in query{
		prev.0 = current.0;
	}
}


fn store_last_translation(
	query:Query<(&mut PreviousTranslation, &PhysicalTranslation)>,
){
	for (mut prev, current) in query{
		prev.0 = current.0;
	}
}

fn interpolate_transform(
  fixed_time: Res<Time<Fixed>>,
	query:Query<
		(
			&mut Transform,
			&PhysicalTranslation, &PreviousTranslation, 
			Option<&PhysicalRotation>, Option<&PreviousRotation>
		), Without<Static>>,
){

	let fraction = fixed_time.overstep_fraction();
	for (mut transform, phys_translation, prev_translation, phys_rotation, prev_rotation) in query{
		
		transform.translation = prev_translation.0.lerp(phys_translation.0, fraction);
		if let Some(phys_rotation) = phys_rotation && let Some(prev_rotation) = prev_rotation{
			transform.rotation =  prev_rotation.0.lerp(phys_rotation.0, fraction)
		}
	}
}


