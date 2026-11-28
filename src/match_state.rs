use bevy::prelude::*;

use crate::{ball::{Ball, PitchSensorMessage}, constants::OPTIMAL_CONTROL_DISTANCE, game_state::GameState, motion::*, physics::EPSILON_TOLERANCE, pitch::SensorType, player::PlayerState, shared::PitchConfiguration, team::{Team, TeamMember}};
 
pub struct MatchStatePlugin;

impl Plugin for MatchStatePlugin{
	fn build(&self, app: &mut App) {
		app
			.init_resource::<MatchState>()
			.init_resource::<BallPressure>()
			.add_systems(OnEnter(GameState::Playing), init_match_state)
			.add_systems(FixedPreUpdate, (update_match_state, calculate_ball_pressure).chain())
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
	players:Query<(Entity, &PhysicalTranslation, &PhysicalRotation, &Velocity, &TeamMember, &PlayerState)>
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

	for (entity, translation, rotation, velocity, team, player_state) in players{
		let snapshot = PlayerSnapshot{ 
			entity, 
			translation:translation.0, 
			velocity: velocity.to_vec3(), 
			can_attack: player_state.can_attack(), 
			can_dribble: player_state.can_dribble(),
    	rotation:rotation.0,	
		};
		if match_state.top_team == Some(team.0){
			match_state.north_team.push(snapshot);
		}
		else{
			match_state.south_team.push(snapshot);
		}
	}
}

fn calculate_ball_pressure(
	match_state: Res<MatchState>,
	mut ball_pressure: ResMut<BallPressure>,
){
	ball_pressure.north = calculate_team_pressure(&match_state.north_team, match_state.ball_location, match_state.ball_velocity);
	ball_pressure.south = calculate_team_pressure(&match_state.south_team, match_state.ball_location, match_state.ball_velocity);
	//info!("ball pressure: north: {} south: {}", ball_pressure.north, ball_pressure.south);
}

const MAX_PRESURE_DISTANCE:f32 = 3.0;
const STANDING_ATTACK_VELOCITY:f32 = 1.5;
//opposition players who can reach the ball in this time trigger maxmimum pressure
const PANIC_TIME:f32 = 0.6;

fn calculate_team_pressure(team:&[PlayerSnapshot], ball_location: Vec3, ball_velocity: Vec3) -> f32 {
	//chillness is 1 to start
	let mut chillness:f32 = 1.;
	for player in team.iter(){
		//players on the floor cannot impart pressure!
		if !player.can_attack{ continue; }
		let diff = ball_location - player.translation;
		if  diff.length_squared() < MAX_PRESURE_DISTANCE * MAX_PRESURE_DISTANCE{
			let (normal, dist) = diff.normalize_and_length();
			let vel_diff = player.velocity - ball_velocity;
			//standing attack velocity is a threat posed by immobile opposition players otherwie it'd be zero
			let approach_speed = vel_diff.dot(normal).max(0.) + STANDING_ATTACK_VELOCITY;
			let time_to_intercept = dist / approach_speed;
			//calculate raw panic
			let panic_raw = (1. - (time_to_intercept / PANIC_TIME));
			//smoooth
			let panic_smooth = panic_raw.smoothstep(0., 1.);
			//and then affect the chill
			chillness *= 1. - panic_smooth;
		}
	}
	//pressure is chillness inverted
	chillness
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

#[derive(Debug, Clone)]
pub struct PlayerSnapshot{
	pub entity:Entity,
	pub translation:Vec3,
	pub rotation:Quat,
	pub velocity:Vec3,
	pub can_attack:bool,
	pub can_dribble:bool,
}

#[derive(Resource, Debug, Default)]
pub struct BallPressure{
	pub north:f32,
	pub south:f32,
}
