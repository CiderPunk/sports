use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::animated_mesh::MeshAnimations;


pub struct FlagPlugin;
impl Plugin for FlagPlugin{
	fn build(&self, app: &mut bevy::app::App) {
	
	}
}


#[derive(AssetCollection, Resource)]
pub struct FlagAssets {
  #[asset(path = "flag.glb#Scene0")]
  pub flag_scene: Handle<WorldAsset>,
	#[asset(path = "flag.glb")]
	pub flag_gltf: Handle<Gltf>,
}

