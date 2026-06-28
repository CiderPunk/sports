use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, transform};
use bevy_asset_loader::prelude::*;
use crate::{assets::AssetLoadState, game_state::GameState, };

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin{
	fn build(&self, app: &mut App) {
		app
			.configure_loading_state(
				LoadingStateConfig::new(AssetLoadState::Startup)
				.load_collection::<PlayerAssets>(),
			)
			.add_systems(OnEnter(GameState::Initialize), (init_player, spawn_players).chain())
			.add_observer(start_player_animation)
			.add_systems(Update, update_active_marker)
			;
	}
}

#[derive(Resource)]
struct PlayerAnimations {
    animations: Vec<AnimationNodeIndex>,
    graph_handle: Handle<AnimationGraph>,
		//scene: Handle<WorldAsset>,
}  

#[derive(Component)]
pub struct Player;

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
  #[asset(path = "player.glb#Material0/std")]
  pub player_material1: Handle<StandardMaterial>,
  #[asset(path = "player.glb#Scene0")]
  pub player_scene: Handle<WorldAsset>,
	#[asset(path = "player.glb")]
  pub player_gltf: Handle<Gltf>,
  #[asset(path = "highlight.glb#Scene0")]
	pub highlight_scene: Handle<WorldAsset>,
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
		player.named_animations["run"].clone(),
		player.named_animations["idle"].clone(),
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
	for i in 0 .. 11{
		let id = commands.spawn((
			Player,
			MeshMaterial3d(player_assets.player_material1.clone()),
			//WorldAssetRoot(player.default_scene.clone().expect("missing default scene")),
			WorldAssetRoot(player_assets.player_scene.clone()),
			Transform::from_xyz((i as f32 * 1.5) - 0.75, 0., -1.),
		)).id();
		if i == 0{
			commands.entity(id).insert(ActivePlayer);
		}
	}

	for i in 0 .. 11{
		commands.spawn((
			Player,
			MeshMaterial3d(player_assets.player_material1.clone()),
			//WorldAssetRoot(player.default_scene.clone().expect("missing default scene")),
			WorldAssetRoot(player_assets.player_scene.clone()),
			Transform::from_xyz((i as f32 * 1.5) - 0.75, 0., 1.).with_rotation(Quat::from_axis_angle(Vec3::Y, PI)),
		));
	}

	//spawn active marker
	commands.spawn((
		ActiveMarker,
		WorldAssetRoot(player_assets.highlight_scene.clone()),
		Transform::from_xyz(0.,0.,0.,),
		Visibility::Hidden,
	));

}

fn start_player_animation(
	event:On<Add, AnimationPlayer>,
	mut commands:Commands,
  mut query: Query<&mut AnimationPlayer>,
	//mut animation_player_query:Query<&mut AnimationPlayer>,
	animations: Res<PlayerAnimations>,
){
	info!("starting animations");
	let Ok(mut anim_player) = query.get_mut(event.entity) else{ 
		return; 
	};
	let mut transitions = AnimationTransitions::new();
	transitions.play(&mut anim_player, animations.animations[1], Duration::ZERO).repeat();

	anim_player.adjust_speeds(1.0);

	commands.entity(event.entity)
		.insert(AnimationGraphHandle(animations.graph_handle.clone()))
		.insert(transitions);
	
}

#[derive(Component)]
pub struct ActivePlayer;


#[derive(Component)]
pub struct ActiveMarker;

fn update_active_marker(
	active_player_query:Query<&GlobalTransform, With<ActivePlayer>>,
	marker:Single<(&mut Transform, &mut Visibility), With<ActiveMarker>>
){
	let (mut transform, mut visible) = marker.into_inner();
	if active_player_query.is_empty() {
		*visible = Visibility::Hidden;
	}
	else{
		for player_transform in active_player_query{
			*transform = Transform::from_translation(player_transform.translation().clone() + Vec3::new(0.,0.5,0.));
			*visible = Visibility::Visible;
		}
	}
}