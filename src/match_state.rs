use bevy::prelude::*;

use crate::{ball::{Ball, PitchSensorMessage}, game_state::GameState, motion::*, pitch::{PitchConfiguration, SensorType}, team::{Team, TeamMember}};
 
pub struct MatchStatePlugin;

impl Plugin for MatchStatePlugin{
	fn build(&self, app: &mut App) {
		app
			.init_resource::<MatchState>()
			.add_systems(OnEnter(GameState::Playing), init_match_state)
			.add_systems(FixedPreUpdate, update_match_state)
			.add_systems(FixedUpdate, handle_pitch_sensors)
		;
	}
}


fn handle_pitch_sensors(
	mut pitch_sensor_reader:MessageReader<PitchSensorMessage>,
	pitch_config:Res<PitchConfiguration>,
	sensor_query:Query<&SensorType>,
){
	for sensor_message in pitch_sensor_reader.read(){
		let Ok(sensor_type) = sensor_query.get(sensor_message.entity) else { continue; };
		match sensor_type{
			SensorType::Side => { 
				info!("Throw in!"); 
			}
			SensorType::End =>  {
				let loc = sensor_message.location;

				if loc.x.abs() < pitch_config.half_goal_width && loc.y < pitch_config.goal_height{
					info!("GOOOOOOAAAAALLLL!!!!");
				}
				else{
					info!("Corner / goal kick");
				}


			}
		}
		
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
	match_state.half_length = pitch_config.half_length ;
	match_state.half_width = pitch_config.half_width;
}


fn update_match_state(
	mut match_state:ResMut<MatchState>,
	time:Res<Time>,
	ball:Single<(&Ball, &Velocity, &PhysicalTranslation)>,
	player_team_query:Query<&TeamMember>,
	players:Query<(Entity, &PhysicalTranslation, &Velocity, &TeamMember)>
){
	match_state.match_time.tick(time.delta());
	let (ball, ball_velocity,  ball_translation) = ball.into_inner();
	match_state.ball_location = ball_translation.0;
	match_state.ball_velocity = ball_velocity.to_vec3();
	match_state.posession = ball.possession
		.and_then(|player|{ player_team_query.get(player).ok()})
		.map(|team| team.0);

	match_state.north_team.clear();
	match_state.south_team.clear();

	for (entity, translation, velocity, team) in players{
		let snapshot = PlayerSnapshot{ entity, translation:translation.0, velocity: velocity.to_vec3() };
		if match_state.top_team == Some(team.0){
			match_state.north_team.push(snapshot);
		}
		else{
			match_state.south_team.push(snapshot);
		}
	}


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
	pub north_team:Vec<PlayerSnapshot>,
	pub south_team:Vec<PlayerSnapshot>,
}

#[derive(Debug, Clone, Copy)]
pub struct PlayerSnapshot{
	pub entity:Entity,
	pub translation:Vec3,
	pub velocity:Vec3,
}
