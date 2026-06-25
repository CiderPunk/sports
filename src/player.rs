use bevy::{gltf::{GltfMaterial, GltfMesh}, prelude::*};

use crate::{assets::{AssetLoadState, GameAssets}, get_gltf_primative};

pub struct PlayerPlugin;




impl Plugin for PlayerPlugin{
	fn build(&self, app: &mut App) {
		app
			.insert_resource(PlayerResources{
				..default()
			})
			.add_systems(OnEnter(AssetLoadState::Loaded), init_player_resources);
	}
}


#[derive(Resource, Default)]
struct PlayerResources{
  player: Handle<Mesh>,
	player_material:Handle<GltfMaterial>,
}

fn init_player_resources(
	mut player_reosources: ResMut<PlayerResources>,
	gltf_assets: Res<Assets<Gltf>>,
  gltf_meshes: Res<Assets<GltfMesh>>,
	game_assets: Res<GameAssets>,
) -> Result<()> {
	let models = gltf_assets.get(&game_assets.models).ok_or("Couldn't get models")?;
	let player_primative = get_gltf_primative!(gltf_meshes, models, "player_mesh" );
	//player_reosources.player = player_primative.mesh.clone();
	//player_reosources.player_material = player_primative.material.clone().ok_or("Missing player material")?;
  Ok(())
}