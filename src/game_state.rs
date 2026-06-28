use bevy::prelude::*;

use crate::assets::AssetLoadState;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin{
	fn build(&self, app: &mut App) {
		app
			.init_state::<GameState>()
			.add_systems(OnEnter(AssetLoadState::Loaded), |mut next_state: ResMut<NextState<GameState>>| {
				info!("Game Initializing");
				next_state.set(GameState::Initialize); 
			})
			.add_systems(OnEnter(GameState::Initialize), |mut next_state: ResMut<NextState<GameState>>| { 
				info!("Game Starting");
				next_state.set(GameState::Playing );
			});
	}
}




#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default, Copy)]
pub enum GameState {
  #[default]
  Loading,
  Initialize,
  Playing,
}