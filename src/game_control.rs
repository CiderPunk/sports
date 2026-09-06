use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use crate::{game_state::GameState, player::{ActivePlayer, PlayerMovement}, team::{PlayerControlled, Team, TeamMember}};
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
struct Kick;


#[derive(Component)]
#[relationship(relationship_target = TeamInputControllers)]
pub struct TeamInputController(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = TeamInputController)]
pub struct TeamInputControllers(Vec<Entity>);


#[derive(Component)]
pub struct GameControl;

fn direction_input_stopped(
	_:On<Complete<MovementInput>>,
	query:Query<&mut PlayerMovement, With<ActivePlayer>>,
){
	//info!("Movement stopped");
	for mut movement in query{
		movement.direction = Vec2::ZERO;
	}
}


fn direction_input_started(
	direction:On<Fire<MovementInput>>,
	context:Query<&TeamInputController>,
	query:Query<(&mut PlayerMovement, &TeamMember), With<ActivePlayer>>,
){
	if let Ok(team) = context.get(direction.context){
		for (mut movement, active_player_team) in query{
			if team.0 == active_player_team.0{
				movement.direction = direction.value;
			}
		}
	}
}



fn kick_started(
	_:On<Fire<Kick>>,
	query:Query<&mut PlayerMovement, With<ActivePlayer>>,
){
	//info!("Movement stopped");
	for mut movement in query{
		movement.direction = Vec2::ZERO;
	}
}


//test
fn spawn_controls(
	teams:Query<Entity, With<PlayerControlled>>,
	mut commands:Commands,

){
	let Some(team_entity) = teams.iter().next() else{
		panic!("No teams found to control");
	};

	info!("GameControl spawned");
	commands.spawn((
		GameControl,
		TeamInputController(team_entity),
		actions!(GameControl[
			(
				Action::<MovementInput>::new(),
				DeadZone::default(),
				Bindings::spawn((Cardinal::wasd_keys(), Axial::left_stick(), Cardinal::dpad())),
			),
			(
				Action::<Kick>::new(),
				bindings![ KeyCode::Space, GamepadButton::South],
			)
		]),
	));
}