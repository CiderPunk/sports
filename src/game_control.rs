use bevy::{math::VectorSpace, prelude::*};
use bevy_enhanced_input::prelude::*;

use crate::{game_state::GameState, player::{ActivePlayer, Movement}};
pub struct GameControlPlugin;

impl Plugin for GameControlPlugin{
	fn build(&self, app: &mut App) {
		app
			.add_input_context::<GameControl>()
			.add_systems(OnEnter(GameState::Initialize), spawn_controls)
			.add_observer(direction_input_started)			
			.add_observer(direction_input_stopped)
			;
	}
}


#[derive(InputAction)]
#[action_output(Vec2)]
struct MovementInput;

#[derive(InputAction)]
#[action_output(bool)]
struct Shoot;


#[derive(InputAction)]
#[action_output(bool)]
struct Pass;

#[derive(Component)]
pub struct GameControl;




fn direction_input_stopped(
	_:On<Complete<MovementInput>>,
	query:Query<&mut Movement, With<ActivePlayer>>,
){
	//info!("Movement stopped");
	for mut movement in query{
		movement.direction = Vec2::ZERO;
	}
}


fn direction_input_stopped(
	_:On<Complete<MovementInput>>,
	query:Query<&mut Movement, With<ActivePlayer>>,
){
	//info!("Movement stopped");
	for mut movement in query{
		movement.direction = Vec2::ZERO;
	}
}


fn direction_input_started(
	direction:On<Fire<MovementInput>>,
	query:Query<&mut Movement, With<ActivePlayer>>,
){
	//info!("Movement {}", direction.value);
	for mut movement in query{
		movement.direction = direction.value;
	}
}



fn spawn_controls(
	mut commands:Commands
){
	info!("GameControl spawned");
	commands.spawn((
		GameControl,
		actions!(GameControl[
			(
				Action::<MovementInput>::new(),
				DeadZone::default(),
//				Scale::splat(100.0),
				Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick(), Cardinal::dpad())),
			),
		]),
	));


}