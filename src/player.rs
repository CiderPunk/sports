use bevy::{math::VectorSpace, prelude::*};
use std::{f32::consts::PI, time::Duration };
use bevy::{gltf::GltfMesh, light::NotShadowCaster, prelude::*, time::{Stopwatch, common_conditions::on_timer}, world_serialization::WorldInstanceReady};
use bevy_asset_loader::prelude::*;

use crate::{ animation_manager::AnimationManager, assets::AssetLoadState, ball::Ball, game_schedule::GameSchedule, game_state::GameState, get_gltf_primative, interpolation::{PhysicalRotation, PhysicalTranslation}, kit::{KitConfiguration, KitGenerator}, match_state::MatchState, physics::{Collider, ColliderShape, CylinderTarget, Velocity}, team::{self, PlayerControlled, Team, TeamMember, TeamMembers}, think_distributor::{ThinkNext, Thinker}};

const PLAYER_SPEED: f32 = 10.;
const PLAYER_TURN_SPEED: f32 = 3.0;
const PLAYER_COLLISION_RADIUS:f32 = 0.5;
const PLAYER_RESTITUTION:f32 = 0.6;
pub const PLAYER_HEIGHT:f32 = 1.8;
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
			.add_systems(FixedUpdate, (player_think, do_movement).in_set(GameSchedule::Movement))
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
pub struct ActivePlayer;

#[derive(Component)]
pub struct Position(Vec2);

#[derive(Component)]
pub struct ActiveMarker;


#[derive(Component)]
pub struct GoalKeeper;


#[derive(Component)]
pub struct ThinkTime(Timer);


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
){

	//go for a classic 4-3-3 whatever that is!
	const POSITIONS:[Vec2;11] = [
		Vec2{ x:-0.8, y:0.9},
		Vec2{ x:0., y:0.9},
		Vec2{ x:0.8, y:0.9},
		//midifeld
		Vec2{ x:-0.4, y:0.6},
		Vec2{ x:0.4, y:0.6},
		Vec2{ x:0., y:0.6},
		//defenders
		Vec2{ x:-0.75, y:0.2},
		Vec2{ x:0.75, y:0.2},
		Vec2{ x:-0.25, y:0.2},
		Vec2{ x:0.25, y:0.2},
		//goalie
		Vec2{ x:0., y:0.05},
	];

	for (team_entity, team) in teams_query{

		for i in 0usize .. 11{
			


			let mut kit = team.kit;	
			kit.shirt_number = i as u8 +1;
			if i == 10{
				kit = team.goalie_kit;
			}

			let (facing, z_pos) = match team.top{
				true => (0., -2.),
				false => (PI, 2.),
			};

			let id = commands.spawn((
				Player{ kit },
				PlayerMovement{ direction: Vec2::ZERO, target_rotation:Quat::from_axis_angle(Vec3::Y, PI), kick_timer: Stopwatch::new(), kick:false },
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
				Position(POSITIONS[i]),
				TeamMember(team_entity),
				Thinker,
			))
			.observe(init_player_animations)
			.observe(init_player_skin)
			.id();


			info!("spawned player {}", id);
			//cheat and make north player 1 active for now
			if i == 0 && team.top{
				//commands.entity(id).insert(ActivePlayer);
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



#[derive(Component, Debug, Default)]
pub struct PlayerMovement{
	pub direction:Vec2,
	target_rotation:Quat,
	pub kick:bool,
	pub kick_timer:Stopwatch,
}

impl PlayerMovement{
	pub fn velocity(&self)->Vec3{
		let vel_2d = self.direction * PLAYER_SPEED;
		Vec3::new(vel_2d.x, 0.0, vel_2d.y)
	}
}



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
			transform.translation = player_transform.translation() + Vec3::new(0.,0.,0.);
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
		rotation.0 = rotation.0.rotate_towards(movement.target_rotation, delta * PLAYER_TURN_SPEED * PI);
  	let movement_3d = Vec3::new(movement.direction.x, 0.0, movement.direction.y);

		if let Ok((dir, length)) =  Dir3::new_and_length(movement_3d){
			movement.target_rotation = Quat::from_rotation_arc(Vec3::Z, dir.xyz());
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

const ACTIVE_PLAYER_TRANSITION_DISTANCE:f32 = 20.0;

fn check_active_player(
	mut commands:Commands,
	ball:Single<(&PhysicalTranslation, &Velocity), With<Ball>>,
	teams:Query<&TeamMembers, With<PlayerControlled>>,
	player_query:Query<(&PhysicalTranslation, Option<&ActivePlayer>)>,
){
	//info!("check active");
	let (translation, velocity) = ball.into_inner();
	//position in half a second
	let projected_position = translation.0 + velocity.to_vec3() * 0.5;
	for team_members in &teams{
		let mut transition = true;
		let mut closest_entity:Option<Entity> = None;
		let mut closest_distance = f32::MAX;
		let mut active_player:Option<Entity> = None;
		for &team_member in team_members{

			if let Ok((translation, active)) = player_query.get(team_member){
				let dist = translation.0.distance_squared(projected_position);
				if dist < closest_distance{
					closest_entity = Some(team_member);
					closest_distance = dist;
				}
				if active.is_some(){
					active_player = Some(team_member);
					if dist < ACTIVE_PLAYER_TRANSITION_DISTANCE * ACTIVE_PLAYER_TRANSITION_DISTANCE{
						//looking for new player
						transition = false;
					}
				}
			}
		}
		if transition && closest_entity != active_player{
			//remove old active
			if let Some(old_active) = active_player{
				commands.entity(old_active).remove::<ActivePlayer>();
			};
			if let Some(new_active) = closest_entity{
				commands.entity(new_active).insert(ActivePlayer);
			}
		}
	}
}


const POSITION_DISTANCE_VARIANCE:f32 = 2.5;
const POSITION_REDUCED_SPEED_DISTANCE:f32 = 5.0;
const POSITION_REDUCED_SPEED_FACTOR:f32 = 0.6;


fn player_think(
	match_state:Res<MatchState>,
	players:Query<(&PhysicalTranslation, &TeamMember, &Position, &mut PlayerMovement), (With<Player>, With<ThinkNext>, Without<ActivePlayer>)>,
){
	for (translation, team, position,  mut movement) in players{
		//are we top or bottom
		let top = match_state.top_team == Some(team.0);
		let side_multiplier = if top { 1.} else { -1.};
		
		//are we defending
		let attacking = match_state.posession == Some(team.0);	
		let ball_loc = match_state.ball_location.z;
		let mut position_depth = match_state.half_length + (ball_loc * side_multiplier); 
		if attacking{
			position_depth *= 1.5;
		}
		position_depth = position_depth.clamp(0.2 * match_state.half_length, 1.95 * match_state.half_length);

		let target_position = Vec2::new(position.0.x * match_state.half_width,  ((position_depth * position.0.y)- match_state.half_length)* side_multiplier);
		let diff = target_position - translation.0.xz();
		let dist_squared= diff.length_squared();
		
		if dist_squared > POSITION_DISTANCE_VARIANCE * POSITION_DISTANCE_VARIANCE{
			movement.direction = diff.normalize_or_zero();
			if dist_squared < POSITION_REDUCED_SPEED_DISTANCE * POSITION_REDUCED_SPEED_DISTANCE{
				movement.direction *= POSITION_REDUCED_SPEED_FACTOR;
			}
			//movement.target_angle = movement.direction.to_angle();
			//info!("moving to target:{} position:{} movement:{} diff:{} attacking:{} top:{}",target_position, position.0, movement.direction, diff, attacking, top );
		}
		else{
			movement.direction = Vec2::ZERO;

			let diff = match_state.ball_location.with_y(0.) - translation.0.with_y(0.);

			movement.target_rotation = Quat::from_rotation_arc(Vec3::Z, diff.normalize_or_zero());
			//movement.target_rotation = Quat::look_at_lh(translation.0, match_state.ball_location, Vec3::Y);
		}


	}
}