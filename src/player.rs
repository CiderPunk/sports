use bevy::prelude::*;
use std::{f32::consts::PI, time::Duration };
use bevy::{gltf::GltfMesh, light::NotShadowCaster, time::{Stopwatch, common_conditions::on_timer}, world_serialization::WorldInstanceReady};
use bevy_asset_loader::prelude::*;
use crate::{ animation_manager::AnimationManager, assets::AssetLoadState, ball::{AfterTouch, Ball}, constants::*, game_control::{ActiveControl, MovementControl}, game_gizmos::{GameGizmoStore, GizmoColour}, game_schedule::GameSchedule, game_state::GameState, get_gltf_primative, helpers::to_nearest_control_point, kit::{KitConfiguration, KitGenerator}, match_state::{MatchState, PlayerSnapshot}, motion::{ChildPhysicalOffset, PhysicalChildren, PhysicalParent, PhysicalRotation, PhysicalTranslation, Velocity}, physics::{ColliderShape::{self, Sphere}, CylinderTarget, EPSILON_TOLERANCE, SphereTarget}, pitch::PitchCollisionZone, shared::{ColliderType, SoccerCollider}, team::{PlayerControlled, Team, TeamMember, TeamMembers}, think_distributor::{ThinkNext, Thinker}};

const PLAYER_SPEED: f32 = 10.;
const PLAYER_SLIDE_SPEED: f32 = 18.;
const PLAYER_SLIDE_RECOVERY_TIME: f32 = 0.5;
const PLAYER_SLIDE_TIME:f32 = 0.6;
const PLAYER_TURN_SPEED: f32 = 3.0;
const PLAYER_COLLISION_RADIUS:f32 = 0.5;
const PLAYER_RESTITUTION:f32 = 0.6;
const SLIDE_COLLIDER_RADIUS:f32 = 0.5;

pub const PLAYER_HEIGHT:f32 = 1.8;
//maximum button hold time for max power
const MAX_POWER_HOLD:f32 = 0.5;
const KICK_RECOVERY_TIME:f32 = 0.3;
const PASS_THRESHOLD:f32 = 0.2;
const MAX_PASS_DISTANCE:f32 = 30.;
const MAX_PASS_DEFLECTION:f32 = PI * 0.125;
const PASS_SPEED:f32 = 20.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin{
	fn build(&self, app: &mut App) {
		app
			.configure_loading_state(
				LoadingStateConfig::new(AssetLoadState::Startup)
				.load_collection::<PlayerAssets>(),
			)
			.add_message::<ActivePlayerSwitch>()
			.add_message::<KickMessage>()
			.add_message::<SlideMessage>()
			.add_message::<BallInteractionMessage>()
			.add_systems(OnEnter(GameState::Initialize), (init_markers, init_player, spawn_players).chain())
			.add_systems(Update, (update_active_marker_position, animate_player))
			.add_systems(FixedUpdate,  (plan_movement, (kick_event_handler, slide_event_handler) ).chain().in_set(GameSchedule::PreMovement))
			.add_systems(FixedUpdate, (switch_active_player, player_think, do_movement).in_set(GameSchedule::Movement))
			.add_systems(Update, (check_active_player).run_if(on_timer(Duration::from_secs_f32(0.2))))
			.add_systems(FixedUpdate, interaction_handler.in_set(GameSchedule::PostMovement))
			
			;
	}
}

#[derive(Component,Debug)]
#[require(MovementControl)]
#[require(TargetRotation)]
#[require(PlayerState)]
pub struct Player{
	kit:KitConfiguration,
}

#[derive(Component)]
pub struct Position(Vec2);

#[derive(Component)]
pub struct ActiveMarker;

#[derive(Component)]
pub struct GoalKeeper;

#[derive(Component)]
pub struct TargetRotation(Quat);

impl Default for TargetRotation{
	fn default() -> Self {
		Self(Quat::from_axis_angle(Vec3::Y,0.))
	}
}

#[derive(Message)]
pub struct ActivePlayerSwitch(Entity);

#[derive(Message)]
pub struct KickMessage{
	entity:Entity,
	kick_time:f32,
}

#[derive(Message)]
pub struct SlideMessage{
	entity:Entity,
	start:bool,
}

#[derive(Message)]
pub struct BallInteractionMessage{
	pub entity:Entity,
}

#[derive(Component, Default, PartialEq, Eq)]
pub enum PlayerState {
  #[default]
	None,
	KickWindup(Stopwatch),
	KickRecover(Timer),
	Slide(Timer),
	SlideRecover(Timer),
}



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



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAnim{
	Idle = 0,
	Run = 1,
	Sprint = 2,
	Slide = 3,
	SlideDown = 4,
	SlideUp = 5,
}

impl From<PlayerAnim> for usize {
	fn from(anim: PlayerAnim) -> Self {
		anim as usize
	}
}


fn init_player(
	mut anim_manager:AnimationManager<Player>,
	player_assets: Res<PlayerAssets>,
){
	info!("Initialize player animations");
	anim_manager.create_graph(player_assets.player_gltf.clone(), &["idle", "run", "sprint", "slide", "slide-down", "slide-up"]);
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
				MovementControl{ direction: Vec2::ZERO, kick_timer: Stopwatch::new(), kick:false },
				WorldAssetRoot(player_assets.player_scene.clone()),
				Transform::default(),
				PhysicalTranslation(Vec3::new((i as f32 * 2.) - 0.75, 0., z_pos)),
				Velocity{ direction: Dir3::Y, speed: 0. },
				PhysicalRotation(Quat::from_rotation_y(facing)),
				Name::new("Player"),
				SoccerCollider{ 
					shape: ColliderShape::Cylinder( CylinderTarget{ 
						direction: Dir3::Y, 
						radius: PLAYER_COLLISION_RADIUS, 
						length: PLAYER_HEIGHT
					}),
					restitution:PLAYER_RESTITUTION,
					filter: PitchCollisionZone::ANY,
				},
				Position(POSITIONS[i]),
				TeamMember(team_entity),
				Thinker,
				PlayerState::None,
			))
			.observe(init_player_animations)
			.observe(init_player_skin)
			.id();
			if i == 10{
				commands.entity(id).insert(GoalKeeper);

			}

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
	anim_manager.attach_animation(event.entity, PlayerAnim::Idle.into());
}


fn update_active_marker_position(
	active_player_query:Query<&GlobalTransform, (With<ActiveControl>, With<Player>)>,
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
	query:Query<(&PlayerState, &Velocity, Entity), With<Player>>,
	//mut animator_query:Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
	mut animation_manager:AnimationManager<Player>,
){
	for (state, velocity, entity) in query{
		match state {
			PlayerState::None => (),
			PlayerState::KickWindup(_) => (),
			PlayerState::KickRecover(_) => (),
			PlayerState::Slide(_) => continue,
			PlayerState::SlideRecover(_) => continue,
		};

		if velocity.speed < EPSILON_TOLERANCE{
			animation_manager.set_animation(entity, PlayerAnim::Idle.into(), 0.2, 1.0, true);
		}
		else{
			animation_manager.set_animation(entity, PlayerAnim::Sprint.into(), 0.2, (velocity.speed / PLAYER_SPEED) .clamp(0.1,1.0), true);
		}
	}
}


fn plan_movement(
	query:Query<(Entity, &mut PlayerState, &mut MovementControl, &mut TargetRotation, &mut Velocity, &PhysicalTranslation), With<Player>>,
	ball:Single<&PhysicalTranslation, With<Ball>>,
	time:Res<Time<Fixed>>,
	mut kick_message_writer:MessageWriter<KickMessage>,
	mut slide_message_writer:MessageWriter<SlideMessage>,
){
	for (entity, mut state,  movement, mut target_rotation, mut velocity, translation) in query{
		match &mut *state{
			PlayerState::None => {
				if movement.kick{	
					if ball.0.distance_squared(translation.0) >4.{
						slide_message_writer.write(SlideMessage { entity, start:true });
						*state = PlayerState::Slide(Timer::from_seconds(PLAYER_SLIDE_TIME, TimerMode::Once))
					}
					else{
						*state = PlayerState::KickWindup(Stopwatch::new());
					}
				}
			},
			PlayerState::KickWindup( stopwatch) => {
				stopwatch.tick(time.delta());
				//unleash the kick?
				if !movement.kick{
					let kick_time = stopwatch.elapsed_secs();
					*state = PlayerState::KickRecover(Timer::from_seconds(KICK_RECOVERY_TIME, TimerMode::Once));
					kick_message_writer.write(KickMessage{ entity, kick_time });					
				}
			},
			PlayerState::KickRecover(timer) =>{
				timer.tick(time.delta());
				if timer.is_finished(){
					*state = PlayerState::None;
				}
			},
			PlayerState::Slide(timer) => {
				timer.tick(time.delta());
				velocity.speed = PLAYER_SLIDE_SPEED * (1.0 - timer.fraction().powf(3.0));
				//info!("speed {}", velocity.speed);
				if timer.is_finished(){
					*state = PlayerState::SlideRecover(Timer::from_seconds(PLAYER_SLIDE_RECOVERY_TIME, TimerMode::Once));
				}
				continue;
			},
			PlayerState::SlideRecover(timer) => {
				slide_message_writer.write(SlideMessage { entity, start:false });
				timer.tick(time.delta());
				if timer.is_finished(){
					*state = PlayerState::None;
				}
				continue;
			},
		}

	//rotation.0 = rotation.0.rotate_towards(target_rotation.0, delta_secs * PLAYER_TURN_SPEED * PI);
		let movement_3d = Vec3::new(movement.direction.x, 0.0, movement.direction.y);
		if let Ok((dir, length)) =  Dir3::new_and_length(movement_3d){
			target_rotation.0 = Quat::from_rotation_arc(Vec3::Z, dir.xyz());
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
	query:Query<(&Velocity, &mut PhysicalTranslation, &mut PhysicalRotation, &TargetRotation), With<Player>>,
		time:Res<Time<Fixed>>,
){
	let delta_secs = time.delta_secs();
	for (velocity, mut translation, mut rotation, target_rotation) in query{
		rotation.0 = rotation.0.rotate_towards(target_rotation.0, delta_secs * PLAYER_TURN_SPEED * PI);
		//TODO: Collision detection!
		translation.0 += velocity.direction * velocity.speed * delta_secs;
	}
}

const ACTIVE_PLAYER_TRANSITION_DISTANCE:f32 = 20.0;

fn check_active_player(
	mut message_writer:MessageWriter<ActivePlayerSwitch>,
	ball:Single<(&PhysicalTranslation, &Velocity), With<Ball>>,
	teams:Query<&TeamMembers, With<PlayerControlled>>,
	player_query:Query<(&PhysicalTranslation, Option<&ActiveControl>)>,
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
		if transition && closest_entity != active_player
			&& let Some(new_active) = closest_entity{
				message_writer.write(ActivePlayerSwitch(new_active));
			}
	}
}



fn switch_active_player(
	mut message_reader:MessageReader<ActivePlayerSwitch>,
	player_query:Query<&TeamMember>,
	previous_player_query:Query<(&TeamMember, Entity),With<ActiveControl>>,
	mut commands:Commands,
){
	for message in message_reader.read(){
		let Ok(team) = player_query.get(message.0) else {
			continue;
		};  
		for (player_team, entity) in previous_player_query{
			if player_team.0 == team.0{
				//switch to same user null-op
				if entity == message.0{ return; }
				commands.entity(entity).remove::<ActiveControl>();
				break;
			}
		}
		commands.entity(message.0).insert(ActiveControl{ team: team.0 });
	}
}

const POSITION_DISTANCE_VARIANCE:f32 = 2.5;
const POSITION_REDUCED_SPEED_DISTANCE:f32 = 5.0;
const POSITION_REDUCED_SPEED_FACTOR:f32 = 0.6;

//AI player think
fn player_think(
	match_state:Res<MatchState>,
	players:Query<(&PhysicalTranslation, &TeamMember, &Position, &mut MovementControl, &mut TargetRotation), (With<Player>, With<ThinkNext>, Without<ActiveControl>)>,
){
	for (translation, team, position,  mut movement, mut target_rotation) in players{
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
			target_rotation.0 = Quat::from_rotation_arc(Vec3::Z, diff.normalize_or_zero());
		}
	}
}


fn slide_event_handler(
	mut commands:Commands,
	mut slide_message_reader:MessageReader<SlideMessage>,
	mut player_query:Query<(&TargetRotation, &mut Velocity), Without<Ball>>,
	child_query:Query<&PhysicalChildren>,
	mut animation_manager:AnimationManager<Player>,
	game_gizmos:Res<GameGizmoStore>,
){
	for message in slide_message_reader.read(){
		let Ok((target_rotation, mut velocity)) = player_query.get_mut(message.entity) else{
			continue;
		};
		let gizmo_handle = game_gizmos.sphere_colours.get(&GizmoColour::Red).expect("missing colour gizmo");

		match message.start{
			true=>{ 
				velocity.direction = Dir3::new_unchecked(target_rotation.0 * Vec3::Z);
				velocity.speed = PLAYER_SLIDE_SPEED;
				animation_manager.set_animation(message.entity, PlayerAnim::Slide.into(), 0.2, 1.0, false);
				commands.spawn((
					Name::new("slide collider"),
					SoccerCollider{ shape: Sphere(SphereTarget{ radius: SLIDE_COLLIDER_RADIUS }), restitution: 0.1, filter: PitchCollisionZone::ANY },
					PhysicalParent(message.entity),
					ChildPhysicalOffset(Vec3::new(0., 0., 0.8)),
					Gizmo{
						handle:gizmo_handle.clone(),
						..default()
					},
					Transform::from_scale(Vec3::splat(SLIDE_COLLIDER_RADIUS)),
					ColliderType::Tackle,
				));
			},
			false=>{ 
				velocity.speed = 0.0;
				animation_manager.set_animation(message.entity, PlayerAnim::SlideUp.into(), 0.2, 1.0, false);
				if let Ok(children) = child_query.get(message.entity){
					for child in children.iter(){
						commands.entity(child).despawn();
					}
				}
			},
		}
	}
}



fn interaction_handler(
	mut message_reader:MessageReader<BallInteractionMessage>,
	collider_query:Query<&PhysicalParent, Without<Ball>>,
	player_query:Query<(&Velocity, &MovementControl), (Without<PhysicalParent>, Without<Ball>)>,
	mut ball_velocity:Single<&mut Velocity, With<Ball>>,
){
	for message in message_reader.read(){
		let Ok(parent_entity) = collider_query.get(message.entity) else { continue;};
		let Ok((velocity, movement)) = player_query.get(parent_entity.0) else{ continue;};
		//let shoot_dir = if movement.direction.length_squared() > EPSILON_TOLERANCE{
		let movement_3d = Vec3::new(movement.direction.x, 0.0, movement.direction.y);
		let shoot_dir = Dir3::new(movement_3d).unwrap_or(velocity.direction);
		let power = velocity.speed / PLAYER_SLIDE_SPEED;
		info!("Tackle! {}", power);

		ball_velocity.direction = shoot_dir;
		ball_velocity.speed = power * 60.0;
	}
}



fn kick_event_handler(
	mut commands:Commands,
	mut kick_message_reader:MessageReader<KickMessage>,
	mut active_player_message_writer:MessageWriter<ActivePlayerSwitch>,
	player_query:Query<(&PhysicalTranslation, &PhysicalRotation, &TargetRotation, &TeamMember), Without<Ball>>,
	ball:Single<(Entity, &PhysicalTranslation, &mut Velocity), With<Ball>>,
	match_state:Res<MatchState>,	
){
	if kick_message_reader.is_empty(){
		return;
	}

	let (ball_entity, ball_translation, mut velocity) = ball.into_inner();
	for message in kick_message_reader.read(){
		let Ok((player_translation, player_rotation, target_rotation, team)) = player_query.get(message.entity) else {
			continue; 
		};
		let Some(to_control_point) = to_nearest_control_point(ball_translation.0, player_translation.0, player_rotation.0) else{
			continue;
		};
	
		//cant kick when ball beyond control distance
		if to_control_point.length_squared() > PLAYER_MAX_CONTROL_DISTANCE * PLAYER_MAX_CONTROL_DISTANCE{
			continue;
		}
		let shoot_dir = target_rotation.0 * Vec3::Z;
		let kick_time = message.kick_time;


		if kick_time > PASS_THRESHOLD{
			// big kick
			let power = (kick_time / MAX_POWER_HOLD).min(1.0);
			info!("Shoot {}", power);
			let shoot = shoot_dir.with_y(power * 0.15).normalize();
			//let shot_dir = Vec3::new(player_movement.direction.x, 0.2, player_movement.direction.y).normalize_or_zero();
			velocity.direction = Dir3::new_unchecked(shoot);
			velocity.speed = power* 60.0;
			commands.entity(ball_entity).insert((
				ActiveControl{ team:team.0 },
				AfterTouch(Timer::from_seconds(0.5, TimerMode::Once))
			));
		}	
		else{
			//pass
			let is_top_team = Some(team.0) == match_state.top_team;
			let pass_candidates = match is_top_team{
				true => &match_state.north_team,
				false => &match_state.south_team,
			};

			let mut best_distance_angle:Option<f32> = None;
			let mut best_pass_candidate:Option<PlayerSnapshot> = None;

			for candidate in pass_candidates{
				//dont pass to yourself that'd be dumb!
				if candidate.entity == message.entity{ continue; }

				let diff = candidate.translation - player_translation.0;
				if diff.length_squared() > MAX_PASS_DISTANCE * MAX_PASS_DISTANCE{ continue; }
				let angle = diff.angle_between(shoot_dir).abs();
				if angle < MAX_PASS_DEFLECTION {
					let distance_angle = (0.2 + angle) * diff.length();
					if best_distance_angle.is_none() || best_distance_angle > Some(distance_angle){
						best_distance_angle = Some(distance_angle);
						best_pass_candidate = Some(*candidate);
					}
				}
			}
			if let Some(candidate) = best_pass_candidate {
				let diff = candidate.translation - player_translation.0;
				let dist = diff.length();
				let eta = dist / PASS_SPEED;
				let predicted_location = candidate.translation + candidate.velocity * eta;
				let shoot_vector = (predicted_location - player_translation.0).normalize();
				velocity.direction = Dir3::new_unchecked(shoot_vector);
				velocity.speed = PASS_SPEED;
				info!("Pass to {} travel time {}", candidate.entity, eta);
				active_player_message_writer.write(ActivePlayerSwitch(candidate.entity));
			}
			else{
				velocity.direction = Dir3::new_unchecked(shoot_dir);
				velocity.speed = PASS_SPEED;
				info!("Pass no target");
				commands.entity(ball_entity).insert((
					ActiveControl{ team:team.0 },
					AfterTouch(Timer::from_seconds(0.5, TimerMode::Once))
				));
			}
			
		}
	}
}


