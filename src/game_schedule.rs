use bevy::prelude::*;

use crate::game_state::GameState;
pub struct GameSchedulePlugin;
impl Plugin for GameSchedulePlugin{
	fn build(&self, app: &mut App) {
		app
			.configure_sets(Update, (
				GameSchedule::EntityUpdates, 
				GameSchedule::MoveEntities,
			).chain()
			.run_if(in_state(GameState::Playing)))
		;
	}
}


#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSchedule{
  EntityUpdates,
  MoveEntities,
}