use bevy::{math::VectorSpace, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{assets::AssetLoadState, game_state::GameState};

pub struct PitchPlugin;
impl Plugin for PitchPlugin{
	fn build(&self, app: &mut App) {
		app
			.insert_resource( 
				PitchConfiguration{
					width:64.,
					length:90.,
					..default()
				}
			)
			.configure_loading_state(
				LoadingStateConfig::new(AssetLoadState::Startup)
				.load_collection::<PitchAssets>(),
			)			
			.add_systems(OnEnter(GameState::Playing), spawn_pitch);
	}
}

#[derive(AssetCollection, Resource)]
pub struct PitchAssets {

  //#[asset(path = "pitch.glb#Scene0")]
  //pub pitch_scene: Handle<WorldAsset>,
	#[asset(path = "pitch.glb#Material0/std")]
	pub pitch_dark_material: Handle<StandardMaterial>,
	#[asset(path = "pitch.glb#Material2/std")]
	pub pitch_light_material: Handle<StandardMaterial>,
	#[asset(path = "pitch.glb#Material3/std")]
	pub pitch_border_material: Handle<StandardMaterial>,
	#[asset(path = "pitch.glb#Material1/std")]
	pub line_material: Handle<StandardMaterial>,
}


fn pitch_segment(half_width:f32, half_length:f32, translation:Vec3, material:Handle<StandardMaterial>)->impl Scene{
	bsn!{
		Mesh3d(asset_value(Plane3d::new(Vec3::Z, Vec2::new(half_width, half_length ) )))
		Transform::from_translation(translation)
		MeshMaterial3d<StandardMaterial>(material)
	}
}

fn pitch_border( mut commands:Commands, half_width:f32, half_length:f32, half_border:f32, material:Handle<StandardMaterial>){
	//top
	commands.spawn_scene(pitch_segment(half_width + (half_border * 2.), half_border,	Vec3::new(0., half_length + half_border, 0.), material.clone()));
	//bottom
	commands.spawn_scene(pitch_segment(half_width + (half_border * 2.), half_border,	Vec3::new(0., -half_length - half_border, 0.), material.clone()));
	//right
	commands.spawn_scene(pitch_segment(half_border, half_length, Vec3::new(half_width + half_border, 0., 0.), material.clone()));
	//left
	commands.spawn_scene(pitch_segment(half_border, half_length, Vec3::new(-half_width - half_border, 0., 0.), material.clone()));
}

fn spawn_pitch(
	mut commands:Commands,
	pitch_assets:Res<PitchAssets>,
	pitch_config:Res<PitchConfiguration>,
){
	pitch_border(commands.reborrow(), 0.5*pitch_config.width,0.5*pitch_config.length,0.5* pitch_config.border, pitch_assets.pitch_border_material.clone());

	let stripe_width = pitch_config.width / pitch_config.stripe_count as f32;
	let start_offset = (-0.5 * pitch_config.width) + (stripe_width * 0.5);
	for n in 0 .. pitch_config.stripe_count{
		commands.spawn_scene(pitch_segment(
			0.5 * stripe_width, 
			0.5 * pitch_config.length, 
			Vec3::new((n as f32 * stripe_width) + start_offset, 0., 0.), 
			match n % 2 {
				0 => pitch_assets.pitch_dark_material.clone(),
				_other => pitch_assets.pitch_light_material.clone(),
			} ));
	}


}



#[derive(Resource)]
pub struct PitchConfiguration{
	width:f32,
	length:f32,
	border:f32,
	stripe_count:u32,
	line_width:f32,
	centre_circle_radius:f32,
	penalty_width:f32,
	penalty_length:f32,
	goal_area_width:f32,
	goal_area_length:f32,
	goal_width:f32,
	corner_arc_radius:f32,
	penalty_spot_dist:f32,
	penalty_arc_radius:f32,
}

impl Default for PitchConfiguration{
	fn default() -> Self {
		Self { 
			width: 64., 
			length: 90.,
			stripe_count:16,
			border:10.,
			centre_circle_radius:9.16,
    	penalty_width: 40.32,
			penalty_length:16.5,
			goal_area_width: 18.32,
			goal_area_length: 5.5,
			goal_width: 7.32,
			corner_arc_radius: 1.,
			penalty_spot_dist: 11.,
			penalty_arc_radius: 9.15,
    	line_width: 0.2,
		}
	}
}



