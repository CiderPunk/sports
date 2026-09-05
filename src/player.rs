use std::{f32::consts::PI, time::Duration };

use bevy::{color::palettes::css::{BLACK, BLUE, BROWN, CORAL, DARK_CYAN, GREEN, GREY, MAGENTA, PINK, PURPLE, RED, WHITE, YELLOW}, gltf::GltfMesh, light::NotShadowCaster, math::VectorSpace, prelude::*, time::{Stopwatch, common_conditions::on_timer}, world_serialization::WorldInstanceReady};
use bevy_asset_loader::prelude::*;

use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;


use rand::seq::IndexedRandom;
use strum::VariantArray;

use crate::{ animation_manager::AnimationManager, assets::AssetLoadState, ball::Ball, game_gizmos::{GameGizmoStore, GizmoColour}, game_schedule::GameSchedule, game_state::GameState, get_gltf_primative, interpolation::{PhysicalRotation, PhysicalTranslation}, kit::{KitColour, KitConfiguration, KitGenerator, KitPattern}, physics::{Collider, ColliderShape, CylinderTarget, Velocity}, team::{Team, TeamMember, TeamSide}};

const PLAYER_SPEED: f32 = 10.;
const PLAYER_TURN_SPEED: f32 = 3.0;
const PLAYER_COLLISION_RADIUS:f32 = 0.5;
const PLAYER_RESTITUTION:f32 = 0.6;
pub const PLAYER_HEIGHT:f32 = 1.8;
pub const PLAYER_MAX_DRIBBLE_DISTANCE:f32 = 1.4;
pub const PLAYER_OPTIMAL_DRIBBLE_DISTANCE:f32 = 0.7;
pub const PLAYER_DRIBBLE_ANGLE:f32 = PI * 0.25;
//pub const PLAYER_DRAW_ANGLE:f32 = PI * 0.5;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin{
	fn build(&self, app: &mut App) {
		app
			.configure_loading_state(
				LoadingStateConfig::new(AssetLoadState::Startup)
				.load_collection::<PlayerAssets>(),
			)
			.add_systems(OnEnter(GameState::Initialize), (init_markers, init_player, spawn_players).chain())
			.add_systems(Update, (update_active_marker_position, animate_player))
			.add_systems(FixedUpdate, plan_movement.in_set(GameSchedule::PreMovement))
			.add_systems(FixedUpdate, do_movement.in_set(GameSchedule::Movement))
			.add_systems(Update, (check_active_player).run_if(on_timer(Duration::from_secs_f32(0.2))))
			;
	}
}


#[derive(Component,Debug)]
#[require(PlayerMovement)]
pub struct Player{
	kit:KitConfiguration,
}

#[derive(Component)]
pub struct TeamNorth;

#[derive(Component)]
pub struct TeamSouth;

#[derive(AssetCollection, Resource, Default)]
pub struct PlayerAssets {
  #[asset(path = "player.glb#Material0/std")]
  pub player_material: Handle<StandardMaterial>,
  #[asset(path = "player.glb#Scene0")]
  pub player_scene: Handle<WorldAsset>,
	#[asset(path = "player.glb")]
  pub player_gltf: Handle<Gltf>,

  #[asset(path = "marker.glb")]
	pub highlight_gltf: Handle<Gltf>,
	#[asset(path = "marker.glb#Material0/std")]
	pub marker_material: Handle<StandardMaterial>,

	pub cone_marker: Option<Handle<Mesh>>,
	pub target_marker: Option<Handle<Mesh>>,

}

fn init_markers(
	mut player_assets:ResMut<PlayerAssets>,
	gltf_assets: Res<Assets<Gltf>>,
  gltf_meshes: Res<Assets<GltfMesh>>,
  //mut meshes: ResMut<Assets<Mesh>>,
) -> Result<()> {
	let markers = gltf_assets.get(&player_assets.highlight_gltf).ok_or("Missing marker asset")?;
  let target_marker_primative = get_gltf_primative!(gltf_meshes, markers, "target_marker" );
	let cone_marker_primative = get_gltf_primative!(gltf_meshes, markers, "cone_marker" );
	player_assets.target_marker = Some(target_marker_primative.mesh.clone());
	player_assets.cone_marker = Some(cone_marker_primative.mesh.clone());
	Ok(())
}


fn init_player(
	mut anim_manager:AnimationManager<Player>,
	player_assets: Res<PlayerAssets>,
){
	info!("Initialize player animations");
	anim_manager.create_graph(player_assets.player_gltf.clone(), &["idle", "run", "sprint"]);
}

fn spawn_players(
	mut commands: Commands,
	teams_query:Query<(Entity, &Team)>,
	player_assets: Res<PlayerAssets>,
	game_gizmos:Res<GameGizmoStore>,
  mut rng: Single<&mut WyRand, With<GlobalRng>>,
){


/*
	let kit_colours = [BLACK, WHITE, RED, GREEN, BLUE, PURPLE, PINK, YELLOW, BROWN, MAGENTA, DARK_CYAN, GREY, CORAL];
	let blue_gizomo = game_gizmos.sphere_colours.get(&GizmoColour::Blue).expect("Missing colour gizmo");
	let red_gizomo = game_gizmos.sphere_colours.get(&GizmoColour::Red).expect("Missing colour gizmo");

	let pink_arrow = game_gizmos.arrow_colours.get(&GizmoColour::Pink).expect("missing pink arrow");
	let blue_arrow = game_gizmos.arrow_colours.get(&GizmoColour::Blue).expect("missing pink arrow");
	let red_arrow = game_gizmos.arrow_colours.get(&GizmoColour::Red).expect("missing pink arrow");
*/

	for (team_entity, team) in teams_query{
		for i in 0 .. 11{
			
			let mut kit = team.kit;	
			kit.shirt_number = i +1;
		
			let (facing, z_pos) = match team.side{
				crate::team::TeamSide::North => (0., -2.),
				crate::team::TeamSide::South => (PI, 2.),
			};

			let id = commands.spawn((
				Player{ kit },
				PlayerMovement{ direction: Vec2::ZERO, target_angle: PI * 1.5 + facing, kick_timer: Stopwatch::new()},
				WorldAssetRoot(player_assets.player_scene.clone()),
				Transform::default(),
				PhysicalTranslation(Vec3::new((i as f32 * 2.) - 0.75, 0., z_pos)),
				Velocity{ direction: Dir3::Y, speed: 0. },
				PhysicalRotation(Quat::from_rotation_y(facing)),
				Name::new("Player"),
				Collider{ 
					shape: ColliderShape::Cylinder( CylinderTarget{ 
						direction: Dir3::Y, 
						radius: PLAYER_COLLISION_RADIUS, 
						length: PLAYER_HEIGHT
					}),
					restitution:PLAYER_RESTITUTION,
				},
				TeamMember(team_entity),
			))
			.observe(init_player_animations)
			.observe(init_player_skin)
			.id();


			info!("spawned player {}", id);
			//cheat and make north player 1 active for now
			if i == 0 && team.side == TeamSide::North{
				commands.entity(id).insert(ActivePlayer);
				info!("Player {}", id);
			}


		}

	}
	

//spawn active marker
	commands.spawn((
		ActiveMarker,
		Mesh3d(player_assets.cone_marker.clone().expect("Cone marker not loaded")),
		MeshMaterial3d(player_assets.marker_material.clone()),
		Transform::from_xyz(0.,0.,0.,),
		Visibility::Hidden,
		NotShadowCaster,
	));

}

fn init_player_skin(
	event:On<WorldInstanceReady>,
	children:Query<&Children>,
	player_query:Query<&Player>,
	material_query:Query<Entity, With<MeshMaterial3d<StandardMaterial>>>,
	mut kit_generator:KitGenerator,
	player_assets: Res<PlayerAssets>,
	mut commands:Commands,
){
	info!("init skin");
	for child in children.iter_descendants(event.entity){
		if let Ok(mesh_entity) = material_query.get(child) 
			&& let Ok(player) = player_query.get(event.entity) {
			let texture_handle = kit_generator.get_or_generate_kit(player.kit);
			let material_handle = kit_generator.make_material(player_assets.player_material.clone(), texture_handle);
			commands.entity(mesh_entity).insert(MeshMaterial3d(material_handle));
			break;
		}
	}
}



fn init_player_animations(
	event:On<WorldInstanceReady>,
	mut anim_manager:AnimationManager<Player>,
){
	anim_manager.attach_animation(event.entity, 0);
}

#[derive(Component)]
pub struct ActivePlayer;

#[derive(Component, Debug, Default)]
pub struct PlayerMovement{
	pub direction:Vec2,
	target_angle:f32,
	kick_timer:Stopwatch,
}

impl PlayerMovement{
	pub fn velocity(&self)->Vec3{
		let vel_2d = self.direction * PLAYER_SPEED;
		Vec3::new(vel_2d.x, 0.0, vel_2d.y)
	}
}




#[derive(Component)]
pub struct ActiveMarker;

fn update_active_marker_position(
	active_player_query:Query<&GlobalTransform, With<ActivePlayer>>,
	marker:Single<(&mut Transform, &mut Visibility), With<ActiveMarker>>,
	time:Res<Time>,
){
	let (mut transform, mut visible) = marker.into_inner();
	if active_player_query.is_empty() {
		*visible = Visibility::Hidden;
	}
	else{
		for player_transform in active_player_query{
			transform.translation = (player_transform.translation() + Vec3::new(0.,0.,0.));
			transform.rotate_local_y(time.delta_secs());
			*visible = Visibility::Visible;
		}
	}
}


fn animate_player(
	query:Query<(&PlayerMovement, Entity), With<Player>>,
	//mut animator_query:Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
	mut animation_manager:AnimationManager<Player>,
){
	for (movement, entity) in query{
		//let Ok((mut player, mut transition)) = animator_query.get_mut(animator.entity) else { continue; };
		if movement.direction == Vec2::ZERO{
			animation_manager.set_animation(entity, 0, 0.2, 1.0, true);
		}
		else{
			animation_manager.set_animation(entity, 2, 0.2, movement.direction.length().clamp(0.1,1.0), true);
		}
	}
}

fn plan_movement(
	query:Query<(&mut PlayerMovement, &mut Velocity, &mut PhysicalRotation), With<Player>>,
	time:Res<Time<Fixed>>,
){
	let delta = time.delta_secs();
	for (mut movement, mut velocity, mut rotation) in query{
		rotation.0 = rotation.0.rotate_towards(Quat::from_axis_angle(Vec3::Y, movement.target_angle + (PI * 0.5)).normalize(), delta * PLAYER_TURN_SPEED *  PI);
		if movement.direction != Vec2::ZERO{
			movement.target_angle = movement.direction.to_angle();
		}
		if let Ok((dir, length)) =  Dir3::new_and_length(Vec3::new(movement.direction.x, 0., -movement.direction.y)){
			velocity.speed = length * PLAYER_SPEED;
			velocity.direction = dir;
		}
		else{
			velocity.direction = Dir3::Y;
			velocity.speed = 0.;
		}	
	}
}

fn do_movement(
	query:Query<(&Velocity, &mut PhysicalTranslation), With<Player>>,
		time:Res<Time<Fixed>>,
){
	let delta = time.delta_secs();
	for (velocity, mut translation) in query{

		//TODO: Collision detection!
		translation.0 += velocity.direction * velocity.speed * delta;
	}
}

fn check_active_player(){




}

