use bevy::{math::VectorSpace, prelude::*};

use crate::{ball::Ball, game_state::GameState, interpolation::PhysicalTranslation, physics::Velocity, team::TeamSide};
 
pub struct MatchStatePlugin;

impl Plugin for MatchStatePlugin{
	fn build(&self, app: &mut App) {
		app
			//.init_resource::<MatchState>()
			.add_systems(OnEnter(GameState::Playing), init_match_state)
			.add_systems(FixedPreUpdate, update_match_state)
		;
	}
}


fn init_match_state(
	mut commands:Commands,
	

){
	commands.insert_resource(MatchState{ 
		match_time: Timer::from_seconds(90., TimerMode::Once), 
		ball_location: Vec3::ZERO, 
		ball_velocity: Vec3::ZERO, 
		posession: TeamSide::Nnoe,
	
	});

}

fn update_match_state(
	mut match_state:ResMut<MatchState>,
	time:Res<Time>,
	ball:Single<(&Velocity, &PhysicalTranslation), With<Ball>>,
){
	match_state.match_time.tick(time.delta());
	let (ball_velocity,  ball_translation) = ball.into_inner();
	match_state.ball_location = ball_translation.0;
	match_state.ball_velocity = ball_velocity.to_vec3();
}


#[derive(Resource, Debug)]
pub struct MatchState{
	match_time:Timer,
	ball_location:Vec3,
	ball_velocity:Vec3,
	posession:TeamSide,
}
