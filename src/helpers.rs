use bevy::prelude::*;
use crate::{ball::BALL_GROUND_LEVEL, constants::*};

pub fn to_nearest_control_point(ball_translation:Vec3, player_translation:Vec3, player_rotation:Quat)->Option<Vec3>{
	let to_ball = ball_translation.xz() - player_translation.xz();
	if to_ball.length_squared() > MAX_INTERACTION_DISTANCE * MAX_INTERACTION_DISTANCE{ return None; }
	let forward = player_rotation * Vec3::Z;
	let forward_2d = forward.xz();
	let angle_to_ball = to_ball.angle_to(forward_2d);
	let target_angle = angle_to_ball.clamp(-MAX_DRIBBLE_ANGLE, MAX_DRIBBLE_ANGLE);
	//nearest control point
	let control_point = player_translation + (forward.rotate_y(target_angle) * OPTIMAL_CONTROL_DISTANCE).with_y(BALL_GROUND_LEVEL); 
	let to_control_point = ball_translation - control_point;
	Some(to_control_point)
}