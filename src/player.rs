use std::time::Duration;

use bevy::prelude::*;
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
	commands.spawn((
		Player,
		MeshMaterial3d(player_assets.player_material1.clone()),
		//WorldAssetRoot(player.default_scene.clone().expect("missing default scene")),
		WorldAssetRoot(player_assets.player_scene.clone()),
		Transform::from_xyz(0., 0., 0.),
	));
	//.observe(start_player_animation);
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
	transitions.play(&mut anim_player, animations.animations[0], Duration::ZERO).repeat();

	anim_player.adjust_speeds(2.0);

	commands.entity(event.entity)
		.insert(AnimationGraphHandle(animations.graph_handle.clone()))
		.insert(transitions);
	
}