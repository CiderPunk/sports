use bevy::{math::VectorSpace, prelude::*, render::render_resource::AsBindGroupShaderType};

use crate::{ball::Ball, game_state::GameState, interpolation::PhysicalTranslation, physics::Velocity, team::{TeamMember, TeamSide}};
 
pub struct MatchStatePlugin;

impl Plugin for MatchStatePlugin{
	fn build(&self, app: &mut App) {
		app
			.init_resource::<MatchState>()
			.add_systems(OnEnter(GameState::Playing), init_match_state)
			.add_systems(FixedPreUpdate, update_match_state)
		;
	}
}


fn init_match_state(
	mut commands:Commands,
		mut match_state:ResMut<MatchState>,
){
	match_state.match_time = Timer::from_seconds(90., TimerMode::Once);

}

fn update_match_state(
	mut match_state:ResMut<MatchState>,
	time:Res<Time>,
	ball:Single<(&Ball, &Velocity, &PhysicalTranslation)>,
	player_team_query:Query<&TeamMember>,

){
	match_state.match_time.tick(time.delta());
	let (ball, ball_velocity,  ball_translation) = ball.into_inner();
	match_state.ball_location = ball_translation.0;
	match_state.ball_velocity = ball_velocity.to_vec3();
	match_state.posession = match ball.possession{
			Some(player_entity) => { 
				if let Ok(team) = player_team_query.get(player_entity) { 
					Some(team.0) 
				} 
				else { 
					None 
				}
			}
			None => None,
	};
}


#[derive(Resource, Debug, Default)]
pub struct MatchState{
	match_time:Timer,
	ball_location:Vec3,
	ball_velocity:Vec3,
	posession:Option<Entity>,
}
