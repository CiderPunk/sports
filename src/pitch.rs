use std::f32::consts::PI;

use bevy::{asset::RenderAssetUsages, color::palettes::css::{RED, WHITE, YELLOW}, gltf::GltfMesh, light::NotShadowCaster, math::VectorSpace, mesh::Indices, prelude::*, };
use bevy_asset_loader::prelude::*;

use crate::{assets::AssetLoadState, game_state::GameState, get_gltf_primative, interpolation::{PhysicalTranslation, Static}, kit::{KitColour, KitConfiguration, KitGenerator, KitPattern}, physics::{Collider, ColliderFilter, ColliderShape, CylinderTarget, PlaneTarget}, shared::{PitchAreaMask, SoccerCollider}};

const LINE_FLOAT_HEIGHT: f32 = 0.05;
const PITCH_RESTITUTION:f32 = 0.65;
const WALL_RESTITUTION:f32 = 0.9;
const POLE_RESTITUTION:f32 = 0.9;
const NET_RESTITUTION:f32 = 0.1;


pub struct PitchPlugin;
impl Plugin for PitchPlugin{
	fn build(&self, app: &mut App) {
		app
			.insert_resource( 
				PitchConfiguration{
					..default()
				}
			)
			.configure_loading_state(
				LoadingStateConfig::new(AssetLoadState::Startup)
				.load_collection::<PitchAssets>(),
			)			
			.add_systems(OnEnter(GameState::Initialize), (modify_materials, init_pitch_models))
			.add_systems(OnEnter(GameState::Playing), (spawn_pitch));

	}
}

#[derive(AssetCollection, Resource, Clone)]
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
  #[asset(path = "pitch_models.glb")]
	pub pitch_models_gltf: Handle<Gltf>,
	#[asset(path = "pitch_models.glb#Material1/std")]
	pub goal_material: Handle<StandardMaterial>,
	#[asset(path = "pitch_models.glb#Material2/std")]
	pub net_material: Handle<StandardMaterial>,
	#[asset(path = "pitch_models.glb#Material0/std")]
	pub flag_material: Handle<StandardMaterial>,

}


#[derive(Resource, Clone)]
pub struct PitchModels{
	pub goal_left:Handle<Mesh>,
	pub goal_right:Handle<Mesh>,
	pub net:Handle<Mesh>,
	pub goal_material:Handle<StandardMaterial>,
	pub net_material:Handle<StandardMaterial>,
	pub flag_material:Handle<StandardMaterial>,
	pub flag:Handle<Mesh>,
	pub flag_pole:Handle<Mesh>,
}

fn init_pitch_models(
	pitch_assets:ResMut<PitchAssets>,
	gltf_assets: Res<Assets<Gltf>>,
  gltf_meshes: Res<Assets<GltfMesh>>,
	mut commands:Commands,
) -> Result<()> {
	let models = gltf_assets.get(&pitch_assets.pitch_models_gltf).ok_or("Missing pitch models")?;
	let goal_left = get_gltf_primative!(gltf_meshes, models, "goal-left").mesh.clone();
	let goal_right = get_gltf_primative!(gltf_meshes, models, "goal-right").mesh.clone();
	let net = get_gltf_primative!(gltf_meshes, models, "net").mesh.clone();
	let flag = get_gltf_primative!(gltf_meshes, models, "flag").mesh.clone();
	let flag_pole = get_gltf_primative!(gltf_meshes, models, "flagpole").mesh.clone();
	commands.insert_resource(
		PitchModels{ 
			goal_left, goal_right, net, 
			goal_material:pitch_assets.goal_material.clone(), 
			net_material:pitch_assets.net_material.clone(), 
			flag,
			flag_pole,
			flag_material:pitch_assets.flag_material.clone(),
		});
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


fn pitch_borders(half_width:f32, half_length:f32, half_border:f32, material:Handle<StandardMaterial>)-> impl SceneList{
	bsn_list!{
		#Top_Border
		NotShadowCaster
		@pitch_segment(half_width + (half_border * 2.), half_border,	Vec3::new(0., 0., half_length + half_border), material.clone())
		--
		#Bottom_Border
		NotShadowCaster
		@pitch_segment(half_width + (half_border * 2.), half_border,	Vec3::new(0., 0.,  -half_length - half_border), material.clone())
		--
		#Right_Border
		NotShadowCaster
		@pitch_segment(half_border, half_length, Vec3::new(half_width + half_border, 0., 0.), material.clone())
		--

		#Left_Border
		NotShadowCaster
		@pitch_segment(half_border, half_length, Vec3::new(-half_width - half_border, 0., 0.), material.clone())
	}
}


fn box_lines(width:f32, length:f32, thickness:f32, material:Handle<StandardMaterial>)-> impl Scene{
	bsn!{
		NotShadowCaster
		Children [
			#Horizontal_Line	
			@line(width, thickness, true, true, material.clone())
			Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, length))
			--
			#Left_Line 
			@line(length, thickness, false, true, material.clone())
			Transform::from_translation(Vec3::new(width * 0.5, LINE_FLOAT_HEIGHT, length * 0.5))
			--
			#Right_Line	
			@line(length, thickness, false, false, material.clone())
			Transform::from_translation(Vec3::new(-width * 0.5, LINE_FLOAT_HEIGHT, length * 0.5))
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

fn pitch_side_colliders(
	pitch_config:PitchConfiguration,
	end:f32,
	end_filter:PitchAreaMask,
) -> impl SceneList{
	let pole_radius = pitch_config.pole_radius;
	let goal_height = pitch_config.goal_height;
	let goal_width = pitch_config.half_goal_width * 2.0;
	let half_goal = pitch_config.half_goal_width;
	let outside_filter = PitchAreaMask::OUTSIDE_GOAL.combine(end_filter);
	let inside_filter = PitchAreaMask::INSIDE_GOAL.combine(end_filter);
	let back = Dir3::from_xyz_unchecked(0.,0.,-1.0 * end);

	info!("inside_filter: {}", inside_filter.0);
	bsn_list!{
		#Right_Post
		SoccerCollider{ 
			shape: ColliderShape::Cylinder(
				CylinderTarget { 
					direction: Dir3::Y,
					radius: pole_radius, 
					length: goal_height, 
				}
			)
			restitution: POLE_RESTITUTION,
			filter: end_filter,
		}
		PhysicalTranslation(Vec3::new(pitch_config.half_goal_width, 0., (end * pitch_config.half_length)))
		Static
		--
		#Left_Post
		SoccerCollider{ 
			shape: ColliderShape::Cylinder(CylinderTarget { 
				direction: Dir3::Y, 
				radius: pole_radius, 
				length: goal_height,
			}),
			restitution: POLE_RESTITUTION,
			filter: end_filter,
		}
		Static
		PhysicalTranslation(Vec3::new(-pitch_config.half_goal_width, 0., end * pitch_config.half_length))
		--
		#Top_Bar
		SoccerCollider{ 
			shape: ColliderShape::Cylinder(CylinderTarget { 
				direction: Dir3::X, 
				radius: pole_radius,
			 	length: goal_width }),
			restitution: POLE_RESTITUTION,
			filter: end_filter,
		}
		Static
		PhysicalTranslation(Vec3::new(-half_goal, pitch_config.goal_height, end * pitch_config.half_length))
		--
		#Goal_Back
		SoccerCollider{
			shape: ColliderShape::Plane(PlaneTarget { normal: back }),
			restitution: NET_RESTITUTION,
			filter: inside_filter,
		}
		Static
		PhysicalTranslation(Vec3::new(0.,0., end * (pitch_config.half_length + pitch_config.goal_depth)))
		--
		#Outside_Goal_Top
		SoccerCollider{ 
			shape: ColliderShape::Cylinder(CylinderTarget { 
				direction: Dir3::X, 
				radius: goal_height,
			 	length: goal_width }),
			restitution: NET_RESTITUTION,
			filter: outside_filter,			
		}
		PhysicalTranslation(Vec3::new(-half_goal,0., end * pitch_config.half_length))
	}
}
/*
fn spawn_pitch_colliders(
	mut commands:Commands,
	pitch_config:Res<PitchConfiguration>,
){	
	let half_pitch_length = pitch_config.half_length;
	let half_total_width = pitch_config.half_width + pitch_config.border;
	let half_total_length = half_pitch_length + pitch_config.border;
	let half_goal = pitch_config.half_goal_width;

	commands.spawn((
		Name::new("Ground"),
		Collider{ 
			shape: crate::physics::ColliderShape::Plane(PlaneTarget{ normal:Dir3::Y,  }),
			restitution: PITCH_RESTITUTION,
			filter: PitchAreaMask::ANY,
		},
		Static,
		//Transform::from_xyz(0.,0.,0.),
		PhysicalTranslation(Vec3::ZERO),
		children![
			//4 walls
			(
				Name::new("left wall"),
				SoccerCollider{ 
					shape: crate::physics::ColliderShape::Plane(PlaneTarget { normal: Dir3::X }),
					restitution: WALL_RESTITUTION,
					filter: PitchAreaMask::ANY,
				},
				PhysicalTranslation(Vec3::new(-half_total_width, 0., 0.)),
				Static,
			),
			(
				Name::new("right wall"),
				SoccerCollider{ 
					shape: crate::physics::ColliderShape::Plane(PlaneTarget { normal: Dir3::NEG_X }),
					restitution: WALL_RESTITUTION,
					filter: PitchAreaMask::ANY,
				},
				PhysicalTranslation(Vec3::new(half_total_width, 0., 0.)),
				Static,
			),
			(
				Name::new("far wall"),
				Collider{ 
					shape: crate::physics::ColliderShape::Plane(PlaneTarget { normal: Dir3::Z }),
					restitution: WALL_RESTITUTION,
					filter: PitchAreaMask::NORTH_GOAL_OUTSIDE,
				},
				PhysicalTranslation(Vec3::new(0., 0., -half_total_length)),
				Static,
			),
			(
				Name::new("near wall"),
				Collider{ 
					shape: crate::physics::ColliderShape::Plane(PlaneTarget { normal: Dir3::NEG_Z }),
					restitution: WALL_RESTITUTION,
					filter: PitchAreaMask::SOUTH_GOAL_OUTSIDE,
				},
				PhysicalTranslation(Vec3::new(0., 0., half_total_length)),
				Static,
			),
			//bottom goal
			(
				Name::new("near right post"),
				Collider{ 
					shape: crate::physics::ColliderShape::Cylinder(CylinderTarget { direction: Dir3::Y, radius: pitch_config.pole_radius, length: pitch_config.goal_height }),
					restitution: POLE_RESTITUTION,
					filter: PitchAreaMask::SOUTH,
				},
				PhysicalTranslation(Vec3::new(half_goal, 0., half_pitch_length)),
				Static,
			),
			(
				Name::new("near left post"),
				Collider{ 
					shape: crate::physics::ColliderShape::Cylinder(CylinderTarget { direction: Dir3::Y, radius: pitch_config.pole_radius, length: pitch_config.goal_height }),
					restitution: POLE_RESTITUTION,
					filter: PitchAreaMask::SOUTH,
				},
				Static,
				PhysicalTranslation(Vec3::new(-half_goal, 0., half_pitch_length)),
			),
			(
				Name::new("near top bar"),
				Collider{ 
					shape: crate::physics::ColliderShape::Cylinder(CylinderTarget { direction: Dir3::X, radius: pitch_config.pole_radius, length: pitch_config.half_goal_width }),
					restitution: POLE_RESTITUTION,
					filter: PitchAreaMask::SOUTH,
				},
				Static,
				PhysicalTranslation(Vec3::new(-half_goal, pitch_config.goal_height, half_pitch_length)),
			),
			(
				Name::new("near goal back"),
				Collider{
					shape: crate::physics::ColliderShape::Plane(PlaneTarget { normal: Dir3::NEG_Z }),
					restitution: NET_RESTITUTION,
					filter: PitchAreaMask::SOUTH_GOAL_INSIDE,
				},
				Static,
				PhysicalTranslation(Vec3::new(0.,0., half_pitch_length + pitch_config.goal_depth)),
			),
			(
				Name::new("near goal left"),
				Collider{
					shape: crate::physics::ColliderShape::Plane(PlaneTarget { normal: Dir3::X}),
					restitution: NET_RESTITUTION,
					filter: PitchAreaMask::SOUTH_GOAL_INSIDE,
				},
				Static,
				PhysicalTranslation(Vec3::new(-pitch_config.half_goal_width,0., 0.)),
			),
			(
				Name::new("near goal right"),
				Collider{
					shape: crate::physics::ColliderShape::Plane(PlaneTarget { normal: Dir3::NEG_X}),
					restitution: NET_RESTITUTION,
					filter: PitchAreaMask::SOUTH_GOAL_INSIDE,
				},
				Static,
				PhysicalTranslation(Vec3::new(pitch_config.half_goal_width,0., 0.)),
			),
			//top goal
			(
				Name::new("far left post"),
				Collider{ 
					shape: crate::physics::ColliderShape::Cylinder(CylinderTarget { direction: Dir3::Y, radius: pitch_config.pole_radius, length: pitch_config.goal_height }),
					restitution: POLE_RESTITUTION,
					filter: PitchAreaMask::NORTH,
				},
				Static,
				PhysicalTranslation(Vec3::new(half_goal, 0., -half_pitch_length))
			),
			(
				Name::new("far right post"),
				Collider{ 
					shape: crate::physics::ColliderShape::Cylinder(CylinderTarget { direction: Dir3::Y, radius: pitch_config.pole_radius, length: pitch_config.goal_height }),
					restitution: POLE_RESTITUTION,
					filter: PitchAreaMask::NORTH,
				},
				PhysicalTranslation(Vec3::new(-half_goal, 0., -half_pitch_length)),
			),
			(
				Name::new("far top bar"),
				Collider{ 
					shape: crate::physics::ColliderShape::Cylinder(CylinderTarget { direction: Dir3::X, radius: pitch_config.pole_radius, length: pitch_config.half_goal_width }),
					restitution: POLE_RESTITUTION,
					filter: PitchAreaMask::NORTH,
				},
				Static,
				PhysicalTranslation(Vec3::new(-half_goal, pitch_config.goal_height, -half_pitch_length)),
			),

		],
	));
}
 */
fn flag_model(
	pitch_models:PitchModels,
	flag_material:Handle<StandardMaterial>,
) -> impl Scene{
	let flag_model = pitch_models.flag.clone();
	let flag_pole = pitch_models.flag_pole.clone();
	let pole_material = pitch_models.net_material.clone();
	bsn!{
		Mesh3d(flag_model)
		MeshMaterial3d<StandardMaterial>(flag_material)
		Children[
			Transform::from_translation(Vec3::ZERO)
			Mesh3d(flag_pole)
			MeshMaterial3d<StandardMaterial>(pole_material)
		]
	}

}

fn goal_model(	
	pitch_models:PitchModels,
	pitch_config:PitchConfiguration,
)-> impl Scene{

	let goal_left = pitch_models.goal_left.clone();
	let goal_right = pitch_models.goal_right.clone();
	let net = pitch_models.net.clone();
	let net_material = pitch_models.net_material.clone();
	let goal_material_1 = pitch_models.goal_material.clone();
	let goal_material_2 = pitch_models.goal_material.clone();
	let goal_height = pitch_config.goal_height;
	let half_goal_width = pitch_config.half_goal_width;

	bsn!{
		Children[

			Transform::from_xyz(-half_goal_width, goal_height, 0.)
			Mesh3d(goal_left)
			MeshMaterial3d<StandardMaterial>(goal_material_1)
			--
			Transform::from_xyz(half_goal_width, goal_height, 0.)
			Mesh3d(goal_right)
			MeshMaterial3d<StandardMaterial>(goal_material_2)
			--
			Transform{
				translation:Vec3::new(0.,goal_height, 0.),
				scale:Vec3::new(half_goal_width, 1., 1.),
			}
			Mesh3d(net)
			MeshMaterial3d<StandardMaterial>(net_material)
		]
	}
}



fn pitch_stripes(
	pitch_assets:PitchAssets,
	pitch_config:PitchConfiguration,
) -> impl SceneList{
	let stripe_width = (pitch_config.half_width * 2.) / pitch_config.stripe_count as f32;
	let start_offset = (-pitch_config.half_width) + (stripe_width * 0.5);
	
	let stripes: Vec<_> = (0 .. pitch_config.stripe_count).map(move |i| {
		pitch_segment(
			0.5 * stripe_width, 
			pitch_config.half_length, 
			Vec3::new((i as f32 * stripe_width) + start_offset, 0., 0.), 
			match i % 2 {
				0 => pitch_assets.pitch_dark_material.clone(),
				_other => pitch_assets.pitch_light_material.clone(),
			}
		)
	}).collect();
	bsn_list!{ {stripes} }
}



fn pitch_side(
	pitch_assets:PitchAssets,
	pitch_models:PitchModels,
	pitch_config:PitchConfiguration,
	flag_material:Handle<StandardMaterial>,
) -> impl Scene{
	let penalty_arc_angle = ((pitch_config.penalty_length - pitch_config.penalty_spot_from_goal) / pitch_config.penalty_arc_radius)
	.clamp(-1., 1.).acos();
	let penalty_spot = pitch_config.half_length - pitch_config.penalty_spot_from_goal;
	bsn!{
		Transform::from_translation(Vec3::ZERO)
		Children[
			#End_Line
			@line(pitch_config.half_width *2., pitch_config.line_width, true, true, pitch_assets.line_material.clone())
			Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, pitch_config.half_length))
			--
			#Right_Side_Line
			@line(pitch_config.half_length * 2., pitch_config.line_width, false, true, pitch_assets.line_material.clone())
			Transform::from_translation(Vec3::new(pitch_config.half_width, LINE_FLOAT_HEIGHT, 0.))
			--
			#Penalty_Box
			@box_lines(pitch_config.penalty_width, -pitch_config.penalty_length, pitch_config.line_width,pitch_assets.line_material.clone())
			Transform::from_translation(Vec3::new(0., 0., pitch_config.half_length))
			--
			#Goal_Box
			@box_lines(pitch_config.goal_area_width, -pitch_config.goal_area_length, pitch_config.line_width, pitch_assets.line_material.clone())
			Transform::from_translation(Vec3::new(0., 0., pitch_config.half_length))
			--
			#Penalty_Arc
			@arc(pitch_config.penalty_arc_radius, pitch_config.line_width, (1.5 * PI) -penalty_arc_angle, 2. * penalty_arc_angle, 24, pitch_assets.line_material.clone())
			Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, penalty_spot + (pitch_config.line_width * 0.5)))
			--
			#Penalty_Spot
			@spot(0.5, pitch_assets.spot_material.clone())
			Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, penalty_spot))
			--
			#Left_Corner_Arc
			@arc(pitch_config.corner_arc_radius, pitch_config.line_width, 1.5*PI, 0.5 * PI, 8, pitch_assets.line_material.clone())
			Transform::from_translation(Vec3::new(-pitch_config.half_width, LINE_FLOAT_HEIGHT, pitch_config.half_length))
			--
			#Right_Corner_Arc
			@arc(pitch_config.corner_arc_radius, pitch_config.line_width, PI, 0.5 * PI, 8, pitch_assets.line_material.clone())
			Transform::from_translation(Vec3::new(pitch_config.half_width, LINE_FLOAT_HEIGHT, pitch_config.half_length))
			--
			#Goal_Model
			Transform::from_xyz(0.,0., pitch_config.half_length).with_rotation(Quat::from_axis_angle(Vec3::Y, PI))
			@goal_model(pitch_models.clone(), pitch_config.clone())		
			--
			#Left_Flag_Model
			Transform::from_xyz(-pitch_config.half_width,0.,pitch_config.half_length)
			@flag_model(pitch_models.clone(), flag_material.clone())
			--
			#Right_Flag_Model
			Transform::from_xyz(pitch_config.half_width,0.,pitch_config.half_length)
			@flag_model(pitch_models.clone(), flag_material.clone())
		]
	}
}

fn build_pitch(
	pitch_assets:PitchAssets,
	pitch_models:PitchModels,
	pitch_config:PitchConfiguration,
	flag_material:Handle<StandardMaterial>,
) -> impl Scene{
	let half_total_width  = pitch_config.half_width + pitch_config.border;
	let half_total_length  = pitch_config.half_length + pitch_config.border;
	bsn!{
		#Pitch
		Transform::from_translation(Vec3::ZERO)
		Children[
			{ pitch_borders(pitch_config.half_width,pitch_config.half_length,0.5* pitch_config.border, pitch_assets.pitch_border_material.clone()) }
			--
			{ pitch_stripes(pitch_assets.clone(), pitch_config.clone()) }
			--
			#Centre_Line
			@line(pitch_config.half_width * 2., pitch_config.line_width, true, true, pitch_assets.line_material.clone())
			Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, 0.))
			--
			#Centre_Circle
			@arc(pitch_config.centre_circle_radius, pitch_config.line_width, 0., 2. * PI, 64, pitch_assets.line_material.clone())
			Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT, 0.))
			--
			#Centre_Spot
			@spot(0.5, pitch_assets.spot_material.clone())
			Transform::from_translation(Vec3::new(0., LINE_FLOAT_HEIGHT,0.))
			--
			#Near_side
			@pitch_side(pitch_assets.clone(), pitch_models.clone(), pitch_config.clone(), flag_material.clone())			
			--
			#Far_side
			@pitch_side(pitch_assets.clone(), pitch_models.clone(), pitch_config.clone(), flag_material.clone())
			Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, PI))		
			--
			#Ground_Collider
			SoccerCollider{ 
				shape: ColliderShape::Plane(PlaneTarget{ normal:Dir3::Y }),
				restitution: PITCH_RESTITUTION,
				filter: PitchAreaMask::ANY,
			}
			Transform::from_translation(Vec3::ZERO)
			Static
			PhysicalTranslation(Vec3::ZERO)
			--
			#Left_Wall
			SoccerCollider{ 
				shape: ColliderShape::Plane(PlaneTarget { normal: Dir3::NEG_X }),
				restitution: WALL_RESTITUTION,
				filter: PitchAreaMask::ANY,
			}
			Static
			PhysicalTranslation(Vec3::new(half_total_width, 0., 0.))
			--		
			#Right_Wall
			SoccerCollider{ 
				shape: ColliderShape::Plane(PlaneTarget { normal: Dir3::X }),
				restitution: WALL_RESTITUTION,
				filter: PitchAreaMask::ANY,
			}
			Static
			PhysicalTranslation(Vec3::new(-half_total_width, 0., 0.))
			--
			#North_Wall
			SoccerCollider{ 
				shape: ColliderShape::Plane(PlaneTarget { normal: Dir3::Z }),
				restitution: WALL_RESTITUTION,
				filter: PitchAreaMask::NORTH_OUTSIDE_GOAL,
			}
			PhysicalTranslation(Vec3::new(0., 0., -half_total_length))
			Static
			--
			#South_Wall
			SoccerCollider{ 
				shape: ColliderShape::Plane(PlaneTarget { normal: Dir3::NEG_Z }),
				restitution: WALL_RESTITUTION,
				filter: PitchAreaMask::SOUTH_OUTSIDE_GOAL,
			}
			PhysicalTranslation(Vec3::new(0., 0., half_total_length))
			Static
			--
			#Inside_Goal_Left
			SoccerCollider{
				shape: ColliderShape::Plane(PlaneTarget { normal: Dir3::X}),
				restitution: NET_RESTITUTION,
				filter: PitchAreaMask::INSIDE_GOAL,
			}
			Static
			PhysicalTranslation(Vec3::new(-pitch_config.half_goal_width,0., 0.))
			--
			#Inside_Goal_Right
			SoccerCollider{
				shape: ColliderShape::Plane(PlaneTarget { normal: Dir3::NEG_X}),
				restitution: NET_RESTITUTION,
				filter: PitchAreaMask::INSIDE_GOAL,
			}
			Static
			PhysicalTranslation(Vec3::new(pitch_config.half_goal_width,0., 0.))
			--
			#Outside_Goal_Left
			SoccerCollider{
				shape: ColliderShape::Plane(PlaneTarget { normal: Dir3::NEG_X}),
				restitution: NET_RESTITUTION,
				filter: PitchAreaMask::OUTSIDE_GOAL,
			}
			Static
			PhysicalTranslation(Vec3::new(-pitch_config.half_goal_width,0., 0.))
			--
			#Outside_Goal_Right
			SoccerCollider{
				shape: ColliderShape::Plane(PlaneTarget { normal: Dir3::X}),
				restitution: NET_RESTITUTION,
				filter: PitchAreaMask::OUTSIDE_GOAL,
			}
			Static
			PhysicalTranslation(Vec3::new(pitch_config.half_goal_width,0., 0.))

			--
			#Goal_Top
			SoccerCollider{
				shape: ColliderShape::Plane(PlaneTarget { normal: Dir3::NEG_Y }),
				restitution: NET_RESTITUTION,
				filter: PitchAreaMask::INSIDE_GOAL,
			}
			Static
			PhysicalTranslation(Vec3::new(0.,pitch_config.goal_height,0.))
			--
			{ pitch_side_colliders(pitch_config.clone(), 1., PitchAreaMask::SOUTH) }	
			--
			{ pitch_side_colliders(pitch_config.clone(), -1., PitchAreaMask::NORTH) }	
		

		]
	}
}


fn make_flag_material(
	mut kit_gen:KitGenerator,
	pitch_models:PitchModels,
) -> Handle<StandardMaterial>{

	let kit = KitConfiguration{ 
		pattern: KitPattern::Quatered,
		colour_primary: KitColour::from_srgba(RED), 
		colour_secondary:KitColour::from_srgba(WHITE),
		colour_tertiary:KitColour::from_srgba(YELLOW),
		shirt_number: 0, 
	};
	let flag_tex = kit_gen.get_or_generate_flag(kit);
	kit_gen.make_material(pitch_models.flag_material.clone(), flag_tex)
}

fn spawn_pitch(
	mut commands:Commands,
	pitch_assets:Res<PitchAssets>,
	pitch_config:Res<PitchConfiguration>,
	pitch_models:Res<PitchModels>,
 	kit_gen:KitGenerator,
){
	let flag_material = make_flag_material(kit_gen, pitch_models.clone());
	commands.spawn_scene(build_pitch(pitch_assets.clone(),pitch_models.clone(), pitch_config.clone(), flag_material));

info!("test: {}", PitchAreaMask::SOUTH.combine(PitchAreaMask::OUTSIDE_GOAL).combine(PitchAreaMask::ANY).contains(PitchAreaMask::SOUTH.combine(PitchAreaMask::INSIDE_GOAL)));
}

#[derive(Resource, Clone)]
pub struct PitchConfiguration{
	pub half_width:f32,
	pub half_length:f32,
	border:f32,
	stripe_count:u32,
	line_width:f32,
	centre_circle_radius:f32,
	penalty_width:f32,
	penalty_length:f32,
	goal_area_width:f32,
	goal_area_length:f32,
	pub half_goal_width:f32,
	pub goal_height:f32,
	pub goal_depth:f32,
	corner_arc_radius:f32,
	pub penalty_spot_from_goal:f32,
	penalty_arc_radius:f32,
	pole_radius:f32,
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



