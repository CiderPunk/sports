use std::f32::consts::PI;

use bevy::{asset::RenderAssetUsages, math::VectorSpace, mesh::Indices, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::{assets::AssetLoadState, game_state::GameState};

const LINE_FLOAT_HEIGHT: f32 = 0.1;

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
		Mesh3d(asset_value(Plane3d::new(Vec3::Y, Vec2::new(half_width, half_length ) )))
		Transform::from_translation(translation)
		MeshMaterial3d<StandardMaterial>(material)
	}
}



fn line(
	length:f32, 
	width:f32, horizontal:bool, bevel_side:bool, material:Handle<StandardMaterial>)->impl Scene{
	let hl = length * 0.5; 
	let hw = width * 0.5;
	let bevel = match bevel_side{
		true => -hw,
		false => hw,
	};

	let verticies = match horizontal{
		true => vec![
						[-hl - bevel, 0., -hw],
						[hl + bevel, 0., -hw],
						[hl - bevel, 0., hw],
						[-hl + bevel, 0., hw],
					],
		false => vec![
						[-hw, 0.,-hl - bevel],
						[-hw, 0., hl + bevel],
						[hw, 0., hl - bevel],
						[hw, 0., -hl + bevel],
					],

	};


	bsn!{
		Mesh3d(asset_value(
			Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default())
				.with_inserted_attribute( //TODO: this should be a trapezium
					Mesh::ATTRIBUTE_POSITION, 
					verticies,
				)
				.with_inserted_attribute( //UV to stretch our line texture the length
					Mesh::ATTRIBUTE_UV_0,
					vec![
						[0.,0.],
						[length,0.],
						[length,1.],
						[0.,1.],
					]
				)
				.with_inserted_indices(Indices::U32(vec![
    			0,1,2,0,2,3,
  			]))
			)
		)
		MeshMaterial3d<StandardMaterial>(material)
	}
}


fn pitch_border_list(half_width:f32, half_length:f32, half_border:f32, material:Handle<StandardMaterial>)-> impl SceneList{
	bsn_list![
		//top
		pitch_segment(half_width + (half_border * 2.), half_border,	Vec3::new(0., 0., half_length + half_border), material.clone()),
		//bottom
		pitch_segment(half_width + (half_border * 2.), half_border,	Vec3::new(0., 0.,  -half_length - half_border), material.clone()),
		//right
		pitch_segment(half_border, half_length, Vec3::new(half_width + half_border, 0., 0.), material.clone()),
		//left
		pitch_segment(half_border, half_length, Vec3::new(-half_width - half_border, 0., 0.), material.clone()),
	]
}


fn box_lines(width:f32, length:f32, thickness:f32, material:Handle<StandardMaterial>)-> impl Scene{
	bsn!{
		
		Children [
			line(width, thickness, true, true, material.clone())
			Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, length)),

			line(length, thickness, false, true, material.clone())
			Transform::from_translation(Vec3::new(width * 0.5, LINE_FLOAT_HEIGHT, length * 0.5)),

			line(length, thickness, false, false, material.clone())
			Transform::from_translation(Vec3::new(-width * 0.5, LINE_FLOAT_HEIGHT, length * 0.5)),
		]
	}
}


fn spawn_pitch(
	mut commands:Commands,
	pitch_assets:Res<PitchAssets>,
	pitch_config:Res<PitchConfiguration>,
){
	let half_width = pitch_config.width * 0.5;
	let half_length = pitch_config.length * 0.5;


	commands.spawn_scene_list(	
		pitch_border_list(half_width,half_length,0.5* pitch_config.border, pitch_assets.pitch_border_material.clone())
	);
	let stripe_width = pitch_config.width / pitch_config.stripe_count as f32;
	let start_offset = (-half_width) + (stripe_width * 0.5);
	for n in 0 .. pitch_config.stripe_count{
		commands.spawn_scene(pitch_segment(
			0.5 * stripe_width, 
			half_length, 
			Vec3::new((n as f32 * stripe_width) + start_offset, 0., 0.), 
			match n % 2 {
				0 => pitch_assets.pitch_dark_material.clone(),
				_other => pitch_assets.pitch_light_material.clone(),
			} 
		));
	}
	let line_material = pitch_assets.line_material.clone();

	let flip = Quat::from_axis_angle(Vec3::Y, PI);


	commands.spawn_scene_list(bsn_list![
		//centre
		line(pitch_config.width, pitch_config.line_width, true, true, line_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, 0.)),
		//bottom
		line(pitch_config.width, pitch_config.line_width, true, true, line_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, half_length)),
		//top
		line(pitch_config.width, pitch_config.line_width, true, true, line_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, -half_length)),
		//right
		line(pitch_config.length, pitch_config.line_width, false, false, line_material.clone())
		Transform::from_translation(Vec3::new(half_width, LINE_FLOAT_HEIGHT, 0.)),
		//left
		line(pitch_config.length, pitch_config.line_width, false, false, line_material.clone())
		Transform::from_translation(Vec3::new(-half_width, LINE_FLOAT_HEIGHT, 0.)),
		//top penalty box
		box_lines(pitch_config.penalty_width, pitch_config.penalty_length, pitch_config.line_width,line_material.clone())
		Transform::from_translation(Vec3::new(0., 0., -half_length)),
		//bottom penalty box
		box_lines(pitch_config.penalty_width, pitch_config.penalty_length, pitch_config.line_width,line_material.clone())
		Transform{ translation: Vec3::new(0.,0.,half_length), rotation:flip },
		//top goal box
		box_lines(pitch_config.goal_area_width, pitch_config.goal_area_length, pitch_config.line_width,line_material.clone())
		Transform::from_translation(Vec3::new(0., 0., -half_length)),
		//bottom goal box
		box_lines(pitch_config.goal_area_width, pitch_config.goal_area_length, pitch_config.line_width,line_material.clone())
		Transform{ translation: Vec3::new(0.,0.,half_length), rotation:flip },


	]);
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



