use std::f32::consts::PI;

use bevy::{asset::RenderAssetUsages, color::palettes::css::PINK, gltf::GltfMesh, light::{NotShadowCaster, NotShadowReceiver}, math::VectorSpace, mesh::Indices, prelude::*, transform};
use bevy_asset_loader::prelude::*;

use crate::{assets::AssetLoadState, game_state::GameState, get_gltf_primative};

const LINE_FLOAT_HEIGHT: f32 = 0.05;

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
			.add_systems(OnEnter(GameState::Initialize), (modify_materials, init_pitch_models))
			.add_systems(OnEnter(GameState::Playing), (spawn_pitch, spawn_pitch_models));
	}
}

#[derive(AssetCollection, Resource)]
pub struct PitchAssets {
  //#[asset(path = "pitch.glb#Scene0")]
  //pub pitch_scene: Handle<WorldAsset>,
	#[asset(path = "pitch.glb#Material3/std")]
	pub pitch_dark_material: Handle<StandardMaterial>,
	#[asset(path = "pitch.glb#Material2/std")]
	pub pitch_light_material: Handle<StandardMaterial>,
	#[asset(path = "pitch.glb#Material1/std")]
	pub pitch_border_material: Handle<StandardMaterial>,
	#[asset(path = "pitch.glb#Material4/std")]
	pub line_material: Handle<StandardMaterial>,
	#[asset(path = "pitch.glb#Material0/std")]
	pub spot_material: Handle<StandardMaterial>,	
  #[asset(path = "goal.glb")]
	pub goal_gltf: Handle<Gltf>,
	#[asset(path = "goal.glb#Material0/std")]
	pub goal_material: Handle<StandardMaterial>,
	#[asset(path = "goal.glb#Material1/std")]
	pub net_material: Handle<StandardMaterial>,

}



#[derive(Resource)]
pub struct PitchModels{
	pub goal_left:Handle<Mesh>,
	pub goal_right:Handle<Mesh>,
	pub net:Handle<Mesh>,
	pub goal_material:Handle<StandardMaterial>,
	pub net_material:Handle<StandardMaterial>,
}

fn init_pitch_models(
	pitch_assets:ResMut<PitchAssets>,
	gltf_assets: Res<Assets<Gltf>>,
  gltf_meshes: Res<Assets<GltfMesh>>,
	mut commands:Commands,
) -> Result<()> {
	let models = gltf_assets.get(&pitch_assets.goal_gltf).ok_or("Missing pitch models")?;
	let goal_left = get_gltf_primative!(gltf_meshes, models, "goal-left").mesh.clone();
	let goal_right = get_gltf_primative!(gltf_meshes, models, "goal-right").mesh.clone();
	let net = get_gltf_primative!(gltf_meshes, models, "net").mesh.clone();
	commands.insert_resource(PitchModels{ goal_left, goal_right, net, goal_material:pitch_assets.goal_material.clone(), net_material:pitch_assets.net_material.clone() });
	Ok(())
}


fn pitch_segment(half_width:f32, half_length:f32, translation:Vec3, material:Handle<StandardMaterial>)->impl Scene{
	bsn!{
		#Pitch_Segment
		Mesh3d(asset_value(Plane3d::new(Vec3::Y, Vec2::new(half_width, half_length ) )))
		Transform::from_translation(translation)
		MeshMaterial3d<StandardMaterial>(material)
		NotShadowCaster 
	}
}

fn arc(
	radius:f32,
	thickness:f32,
	start_angle:f32,
	total_angle:f32,
	segments:u32, 
	material:Handle<StandardMaterial>,
) -> impl Scene {
	let mut verticies = Vec::new();
	let mut uvs = Vec::new();
	let mut indices = Vec::new();
	let segment_angle = total_angle / segments as f32;
	let arc_length = radius * total_angle;
	let segment_uv_length = arc_length / segments as f32;
	let outer_tradius = radius + thickness;

	for i in 0 ..=segments{
		let angle = start_angle + (i as f32 * segment_angle);
		let cos = angle.cos();
		let sin = angle.sin();
		verticies.push([radius * cos, 0., radius * sin]);
		verticies.push([outer_tradius* cos, 0., outer_tradius* sin]);

		let current_u = segment_uv_length * i as f32;
		uvs.push([current_u, 0.]);
		uvs.push([current_u, 1.]);

		if i < segments{
			let current_idx = i * 2;
			indices.extend_from_slice(
				&([
					current_idx, current_idx + 1, current_idx + 2,
					current_idx + 2, current_idx + 1, current_idx + 3,
				])
			);	
		}
	}
	bsn!{
		Mesh3d(asset_value(
			Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::default())
					.with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verticies)
					.with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
					.with_inserted_indices(Indices::U32(indices))
			))
		MeshMaterial3d<StandardMaterial>(material)
		NotShadowCaster 

	}
}


fn spot(size:f32, material:Handle<StandardMaterial>) -> impl Scene{
	bsn!{
		Mesh3d(asset_value(Plane3d::new(Vec3::Y, Vec2::new(0.5 * size, 0.5 * size))))
		MeshMaterial3d<StandardMaterial>(material)
		NotShadowCaster 

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
				.with_inserted_attribute( 
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
		#Top_Border
		pitch_segment(half_width + (half_border * 2.), half_border,	Vec3::new(0., 0., half_length + half_border), material.clone()),
		//bottom
		#Bottom_Border
		pitch_segment(half_width + (half_border * 2.), half_border,	Vec3::new(0., 0.,  -half_length - half_border), material.clone()),
		//right
		#Right_Border
		pitch_segment(half_border, half_length, Vec3::new(half_width + half_border, 0., 0.), material.clone()),
		//left
		#Left_Border
		pitch_segment(half_border, half_length, Vec3::new(-half_width - half_border, 0., 0.), material.clone()),
	]
}


fn box_lines(width:f32, length:f32, thickness:f32, material:Handle<StandardMaterial>)-> impl Scene{
	bsn!{
		
		NotShadowCaster 

		Children [
			#Horizontal_Line
			line(width, thickness, true, true, material.clone())
			Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, length)),


			#Left_Line
			line(length, thickness, false, true, material.clone())
			Transform::from_translation(Vec3::new(width * 0.5, LINE_FLOAT_HEIGHT, length * 0.5)),


			#Right_Line
			line(length, thickness, false, false, material.clone())
			Transform::from_translation(Vec3::new(-width * 0.5, LINE_FLOAT_HEIGHT, length * 0.5)),
		]
	}
}

fn modify_materials(
	pitch_assets:ResMut<PitchAssets>,
	mut materials: ResMut<Assets<StandardMaterial>>,
){
	let alpha_mat_handles = [
		pitch_assets.line_material.clone(),
		pitch_assets.spot_material.clone(),
	];
	for handle in alpha_mat_handles{
		if let Some(mut material) = materials.get_mut(&handle){
			material.alpha_mode = AlphaMode::Add;
		}
	}
}


fn spawn_pitch_models(
	mut commands:Commands,
	pitch_models:Res<PitchModels>,
	pitch_config:Res<PitchConfiguration>,
){


	let half_pitch_length = pitch_config.length * 0.5;

	let lower_goal_transform = Transform::from_xyz(0.,0., half_pitch_length).with_rotation(Quat::from_axis_angle(Vec3::Y, PI));
	commands.spawn_scene_list(

		bsn_list![
			(
				Transform{
					translation:Vec3::new(0.,0.,half_pitch_length),
					rotation:Quat::from_axis_angle(Vec3::Y, PI),
				}
				goal_model(&pitch_models, &pitch_config)
			),
			(
				Transform::from_xyz(0.,0., -half_pitch_length)
				goal_model(&pitch_models, &pitch_config)
			)
		]);
}


fn goal_model(	
	pitch_models:&Res<PitchModels>,
	pitch_config:&Res<PitchConfiguration>,
)-> impl Scene{

	let goal_left = pitch_models.goal_left.clone();
	let goal_right = pitch_models.goal_right.clone();
	let net = pitch_models.net.clone();
	let net_material = pitch_models.net_material.clone();
	let goal_material_1 = pitch_models.goal_material.clone();
	let goal_material_2 = pitch_models.goal_material.clone();
	let goal_height = pitch_config.goal_height;
	let goal_width = pitch_config.goal_width;
	let half_goal_width = goal_width * 0.5;
	bsn!{
		Children[
			(
				Transform::from_xyz(-half_goal_width, goal_height, 0.)
				Mesh3d(goal_left)
				MeshMaterial3d<StandardMaterial>(goal_material_1)
			),
			(
				Transform::from_xyz(half_goal_width, goal_height, 0.)
				Mesh3d(goal_right)
				MeshMaterial3d<StandardMaterial>(goal_material_2)
			),
			(
				Transform{
					translation:Vec3::new(0.,goal_height, 0.),
					scale:Vec3::new(half_goal_width, 1., 1.),
				}
				Mesh3d(net)
				MeshMaterial3d<StandardMaterial>(net_material)
			)
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
	let spot_material = pitch_assets.spot_material.clone();

	let flip = Quat::from_axis_angle(Vec3::Y, PI);
	let penalty_arc_angle = ((pitch_config.penalty_length - pitch_config.penalty_spot_from_goal) / pitch_config.penalty_arc_radius)
		.clamp(-1., 1.).acos();
	let penalty_spot = half_length - pitch_config.penalty_spot_from_goal;
	//info!("penalty arc angle: {}", penalty_arc_angle);

	let half_line_width = 0.25 * pitch_config.line_width;
	commands.spawn_scene_list(bsn_list![


		//centre
		#Centre_Line
		line(pitch_config.width, pitch_config.line_width, true, true, line_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, 0.)),
		//bottom
		#Bottom_Line
		line(pitch_config.width, pitch_config.line_width, true, true, line_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, half_length)),
		//top
		#Top_Line
		line(pitch_config.width, pitch_config.line_width, true, false, line_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, -half_length)),
		//right
		#Right_Side_Line
		line(pitch_config.length, pitch_config.line_width, false, true, line_material.clone())
		Transform::from_translation(Vec3::new(half_width, LINE_FLOAT_HEIGHT, 0.)),
		//left
		#Left_Side_Line
		line(pitch_config.length, pitch_config.line_width, false, false, line_material.clone())
		Transform::from_translation(Vec3::new(-half_width, LINE_FLOAT_HEIGHT, 0.)),
		//top penalty box
		#Top_Penalty_Box
		box_lines(pitch_config.penalty_width, pitch_config.penalty_length, pitch_config.line_width,line_material.clone())
		Transform::from_translation(Vec3::new(0., 0., -half_length)),
		//bottom penalty box
		#Bottom_Penalty_Box
		box_lines(pitch_config.penalty_width, pitch_config.penalty_length, pitch_config.line_width,line_material.clone())
		Transform{ translation: Vec3::new(0.,0.,half_length), rotation:flip },
		//top goal box
		#Top_Goal_Box
		box_lines(pitch_config.goal_area_width, pitch_config.goal_area_length, pitch_config.line_width,line_material.clone())
		Transform::from_translation(Vec3::new(0., 0., -half_length)),
		//bottom goal box
		#Bottom_Goal_Box
		box_lines(pitch_config.goal_area_width, pitch_config.goal_area_length, pitch_config.line_width,line_material.clone())
		Transform{ translation: Vec3::new(0.,0.,half_length), rotation:flip },
		//centre circle
		#Centre_Circle
		arc(pitch_config.centre_circle_radius, pitch_config.line_width, 0., 2. * PI, 64, line_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, 0.)),
		//TL corner
		#Top_Left_Corner_Arc
		arc(pitch_config.corner_arc_radius, pitch_config.line_width, 0., 0.5 * PI, 8, line_material.clone())
		Transform::from_translation(Vec3::new(-half_width, LINE_FLOAT_HEIGHT, -half_length)),
		//TR corner
		#Top_Right_Corner_Arc
		arc(pitch_config.corner_arc_radius, pitch_config.line_width, 0.5*PI, 0.5 * PI, 8, line_material.clone())
		Transform::from_translation(Vec3::new(half_width, LINE_FLOAT_HEIGHT, -half_length)),
		//BR corner
		#Bottom_Right_Corner_Arc
		arc(pitch_config.corner_arc_radius, pitch_config.line_width, 1.0*PI, 0.5 * PI, 8, line_material.clone())
		Transform::from_translation(Vec3::new(half_width, LINE_FLOAT_HEIGHT, half_length)),
		//BL corner
		#Bottom_Left_Corner_Arc
		arc(pitch_config.corner_arc_radius, pitch_config.line_width, 1.5*PI, 0.5 * PI, 8, line_material.clone())
		Transform::from_translation(Vec3::new(-half_width, LINE_FLOAT_HEIGHT, half_length)),
		//top penalty arc
		#Top_Penalty_Arc
		arc(pitch_config.penalty_arc_radius, pitch_config.line_width, (0.5 * PI) -penalty_arc_angle, 2. * penalty_arc_angle, 24, line_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, -penalty_spot - half_line_width)),
		//bottom penalty arc
		#Bottom_Penalty_Arc
		arc(pitch_config.penalty_arc_radius, pitch_config.line_width, (1.5 * PI) -penalty_arc_angle, 2. * penalty_arc_angle, 24, line_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, penalty_spot + half_line_width)),
		//centre spot
		#Centre_Spot
		spot(0.5, spot_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT,0.)),
		//top penalty spot
		#Top_Penalty_Spot
		spot(0.5, spot_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, -penalty_spot)),
		//bottom penalty spot
		#Bottom_Penalty_Spot
		spot(0.5, spot_material.clone())
		Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, penalty_spot)),

	]);
}



#[derive(Resource)]
pub struct PitchConfiguration{
	pub width:f32,
	pub length:f32,
	border:f32,
	stripe_count:u32,
	line_width:f32,
	centre_circle_radius:f32,
	penalty_width:f32,
	penalty_length:f32,
	goal_area_width:f32,
	goal_area_length:f32,
	pub goal_width:f32,
	pub goal_height:f32,
	corner_arc_radius:f32,
	pub penalty_spot_from_goal:f32,
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
			goal_height:2.44,
			corner_arc_radius: 1.,
			penalty_spot_from_goal: 11.,
			penalty_arc_radius: 9.15,
    	line_width: 0.2,
		}
	}
}



