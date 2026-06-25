use  bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::{LoadingState, LoadingStateAppExt, config::ConfigureLoadingState}};


pub struct AssetsPlugin;
impl Plugin for AssetsPlugin{
	fn build(&self, app: &mut App) {
		app
			.init_state::<AssetLoadState>()
			.add_loading_state(
				LoadingState::new(AssetLoadState::Startup)
				.continue_to_state(AssetLoadState::Loaded)
				.load_collection::<GameAssets>()
			);
	}
}


#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Copy)]
pub enum AssetLoadState {
  #[default]
  Startup,
  Loaded,
}



#[derive(AssetCollection, Resource)]
pub struct GameAssets {
  #[asset(path = "models.glb")]
  pub models: Handle<Gltf>,
}

