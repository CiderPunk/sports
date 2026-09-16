use bevy::prelude::*;

use crate::{ball::Ball, game_state::GameState, interpolation::PhysicalTranslation, physics::Velocity, pitch::PitchConfiguration, team::{Team, TeamMember}};
 
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
	mut match_state:ResMut<MatchState>,
	teams:Query<(Entity, &Team)>,
	pitch_config:Res<PitchConfiguration>,
){
	match_state.match_time = Timer::from_seconds(90., TimerMode::Once);
	//get which team is occupying the top of the pitch
	if let Some((entity,_)) =  teams.iter().find(|(_, team)| team.top ){
		match_state.top_team = Some(entity);		
	}
	match_state.half_length = pitch_config.length * 0.5;
	match_state.half_width = pitch_config.width * 0.5;
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
	match_state.posession = ball.possession
		.and_then(|player|{ player_team_query.get(player).ok()})
		.map(|team| team.0);

	/*
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
	 */
}


#[derive(Resource, Debug, Default)]
pub struct MatchState{
	pub match_time:Timer,
	pub ball_location:Vec3,
	pub ball_velocity:Vec3,
	pub posession:Option<Entity>,
	pub top_team:Option<Entity>, //which team is north...
	pub half_length:f32,
	pub half_width:f32,
}
