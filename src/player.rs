use std::{f32::consts::PI, time::Duration};

use bevy::{gltf::GltfMesh, light::NotShadowCaster, prelude::*, world_serialization::WorldInstanceReady};
use bevy_asset_loader::prelude::*;
use crate::{assets::AssetLoadState, ball::BallMotion, colliders::CollisionCylinder, game_schedule::GameSchedule, game_state::GameState, get_gltf_primative};

const PLAYER_SPEED: f32 = 10.;
const PLAYER_INFLUENCE:f32 = 1.5;
const PLAYER_DRIBBLE_CENTRE:f32 = 0.5;

const PLAYER_COLLISION_RADIUS:f32 = 0.5;
const PLAYER_HEIGHT:f32 = 1.8;


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin{
	fn build(&self, app: &mut App) {
		app
			.configure_loading_state(
				LoadingStateConfig::new(AssetLoadState::Startup)
				.load_collection::<PlayerAssets>(),
			)
			.add_systems(OnEnter(GameState::Initialize), (init_markers, init_player, spawn_players).chain())
			//.add_observer(init_player_animations)
			.add_systems(Update, update_active_marker)
			.add_systems(Update, (move_player, animate_player).in_set(GameSchedule::PlayerUpdates))
			//.add_systems(Update, dribble.in_set(GameSchedule::BallUpdate))
			
			;
	}
}



#[derive(Resource)]
struct PlayerAnimations {
	animations: Vec<AnimationNodeIndex>,
	graph_handle: Handle<AnimationGraph>,
		//scene: Handle<WorldAsset>,
}  

#[derive(Component,Debug)]
#[require(Movement)]
pub struct Player;


#[derive(Component)]
pub struct Animator{
	entity:Entity,
}

#[derive(AssetCollection, Resource, Default)]
pub struct PlayerAssets {
  #[asset(path = "player.glb#Material0/std")]
  pub player_material1: Handle<StandardMaterial>,
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
	mut commands: Commands,
	gltfs: Res<Assets<Gltf>>,
	player_assets:Res<PlayerAssets>,
	mut graphs: ResMut<Assets<AnimationGraph>>,
){
	info!("Initialize player animations");
	let player = gltfs.get(&player_assets.player_gltf).expect("Missing player asset");
	//build animatiopn graph
	let (graph, node_indices) = AnimationGraph::from_clips([
		player.named_animations["idle"].clone(),
		player.named_animations["run"].clone(),
		player.named_animations["sprint"].clone(),
	]);
	let graph_handle = graphs.add(graph);
	commands.insert_resource(PlayerAnimations {
		animations: node_indices,
		graph_handle, 
	});
}

fn spawn_players(
	mut commands: Commands,
	player_assets: Res<PlayerAssets>,
){
	for i in 0 .. 1{
		let id = commands.spawn((
			Player,
			MeshMaterial3d(player_assets.player_material1.clone()),
			//WorldAssetRoot(player.default_scene.clone().expect("missing default scene")),
			WorldAssetRoot(player_assets.player_scene.clone()),
			Transform::from_xyz((i as f32 * 3.) - 0.75, 0., -1.),
			CollisionCylinder{ radius: PLAYER_COLLISION_RADIUS, height: PLAYER_HEIGHT }
		)).observe(init_player_animations)
		.id();
		if i == 0{
			commands.entity(id).insert(ActivePlayer);
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

fn init_player_animations(
	event:On<WorldInstanceReady>,
	children_query: Query<&Children>,
	mut anim_player_query: Query<&mut AnimationPlayer>,
	mut commands:Commands,
	animations: Res<PlayerAnimations>,
){
	for descendant in children_query.iter_descendants(event.entity) {
		if let Ok(mut anim_player) = anim_player_query.get_mut(descendant) {
			//info!("Foundanimation player");
			let mut transitions = AnimationTransitions::new();
			transitions.play(&mut anim_player, animations.animations[0], Duration::ZERO).repeat();
			commands.entity(descendant)
				.insert(AnimationGraphHandle(animations.graph_handle.clone()))
				.insert(transitions);
			commands.entity(event.entity).insert(Animator{ entity: descendant});
			break;
		}
	}	
}

#[derive(Component)]
pub struct ActivePlayer;

#[derive(Component, Debug, Default)]
pub struct Movement{
	pub direction:Vec2,
	target_angle:f32,
}

impl Movement{
	pub fn velocity(&self)->Vec3{
		let vel_2d = self.direction * PLAYER_SPEED;
		Vec3::new(vel_2d.x, 0.0, vel_2d.y)
	}

}


#[derive(Component)]
pub struct ActiveMarker;

fn update_active_marker(
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
			transform.translation = (player_transform.translation().clone() + Vec3::new(0.,0.,0.));
			transform.rotate_local_y(time.delta_secs());
			*visible = Visibility::Visible;
		}
	}
}


fn animate_player(
	mut query:Query<(&Movement, &Animator), With<Player>>,
	mut animator_query:Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
	animations:Res<PlayerAnimations>,
){
	for (movement, animator) in query{
		let Ok((mut player, mut transition)) = animator_query.get_mut(animator.entity) else { continue; };
		if movement.direction == Vec2::ZERO{
			if transition.get_main_animation() != Some(animations.animations[0]){
				transition.play(&mut player, animations.animations[0], Duration::from_secs_f32(0.2)).repeat().set_speed(1.);
			}
		}
		else{

			if let Some(active_animation) = player.animation_mut(animations.animations[2]){
				active_animation.set_speed(movement.direction.length().clamp(0.1,1.0));
			}

			if transition.get_main_animation() != Some(animations.animations[2]){
				transition.play(&mut player, animations.animations[2], Duration::from_secs_f32(0.1)).repeat();
			}
		}
	}
}

fn move_player(
	query:Query<(&mut Movement, &mut Transform, ), With<Player>>,
	time:Res<Time>,
){
	for (mut movement, mut transform) in query{
		if movement.direction != Vec2::ZERO{
			movement.target_angle = movement.direction.to_angle();
		}
		transform.translation += Vec3::new(movement.direction.x, 0., -movement.direction.y) * time.delta_secs() * PLAYER_SPEED;
		transform.rotation = transform.rotation.rotate_towards(Quat::from_axis_angle(Vec3::Y, movement.target_angle + (PI * 0.5)).normalize(), time.delta_secs() * 4.0 *  PI);
	}
}

