
use std::f32::consts::PI;

use bevy::{color::palettes::css::{BLUE, RED, YELLOW}, math::FloatPow, prelude::*};
use bevy_asset_loader::prelude::*;
use crate::{assets::AssetLoadState, game_schedule::GameSchedule, game_state::GameState, interpolation::{PhysicalRotation, PhysicalTranslation}, physics::{Collidable, Collider, ColliderShape, EPSILON_TOLERANCE, FrameMotion, HitResult, PhysicalProperties, SphereSweep, SphereTarget, Velocity}, player::{ PLAYER_DRIBBLE_ANGLE, PLAYER_HEIGHT, PLAYER_MAX_DRIBBLE_DISTANCE, PLAYER_OPTIMAL_DRIBBLE_DISTANCE, Player, PlayerMovement}};


const BALL_SCALE: f32 = 0.5;
pub const BALL_RADIUS:f32 = 0.25 * BALL_SCALE;

const GRAVITY_DOWN:f32 = 9.8;
const GRAVITY:Vec3 = Vec3::new(0., -GRAVITY_DOWN, 0.);
const BALL_RESTITUTION:f32 = 0.8;

//air damping
const DRAG_COEFFICIENT:f32 = 0.30;
const AIR_DENSITY:f32 = 1.225;
const BALL_CROSS_SECTION_AREA:f32 = PI * BALL_RADIUS * BALL_RADIUS; //0.038;
const AIR_DAMPING:f32 = 0.5 * AIR_DENSITY * BALL_CROSS_SECTION_AREA * DRAG_COEFFICIENT;

//ground damping
//const ROLLING_RESISTANCE:f32 = 0.08;
const ROLLING_RESISTANCE:f32 = 0.2;
const BALL_MASS:f32 = 0.43;
//don't need ball mass!
//const GROUND_DECELERATION:f32 = (BALL_MASS * GRAVITY * ROLLING_RESISTANCE) / BALL_MASS;  
const GROUND_DECELERATION:f32 = GRAVITY_DOWN * ROLLING_RESISTANCE;  


pub struct BallPlugin;
impl Plugin for BallPlugin{
	fn build(&self, app: &mut App) {
		app
			.configure_loading_state(
				LoadingStateConfig::new(AssetLoadState::Startup)
				.load_collection::<BallAssets>(),
			)
			.add_systems(OnEnter(GameState::Playing), spawn_ball)
			.add_systems(FixedUpdate, (dribble, physics).chain().in_set(GameSchedule::PreMovement))
			.add_systems(FixedUpdate, collisions.in_set(GameSchedule::Movement))
		//	.add_systems(FixedUpdate, (decide_influence, update_ball).chain().in_set(GameSchedule::MoveBall))
			;
	}
}

#[derive(AssetCollection, Resource)]
pub struct BallAssets {
  #[asset(path = "ball.glb#Material0/std")]
  pub ball_material: Handle<StandardMaterial>,
  #[asset(path = "ball.glb#Scene0")]
  pub ball_scene: Handle<WorldAsset>,
}


#[derive(Component, Debug)]
pub struct Ball{
	//pub velocity:Vec3,
	pub control:Vec3,
	roll_axis:Dir3,
	roll_speed:f32,
	last_touch:Option<Entity>,
}



fn spawn_ball(
	mut commands:Commands,
	ball_assets:Res<BallAssets>,
){
	commands.spawn((
		WorldAssetRoot(ball_assets.ball_scene.clone()),
		//Transform::from_translation(Vec3::new(0., BALL_GROUND_LEVEL ,0.)).with_scale(Vec3::splat(BALL_SCALE)),
		Transform::from_translation(Vec3::new(-30., 10. ,0.)).with_scale(Vec3::splat(BALL_SCALE)),
		Ball{
			..default()
		},
		Collider{ 
			shape: ColliderShape::Sphere( SphereTarget{ radius: BALL_RADIUS }),
			restitution:BALL_RESTITUTION,
		},
		Velocity{ direction: Dir3::X, speed:5. },
		PhysicalTranslation(Vec3::new(-30., 10. ,0.)),
	));
}

pub const DRIBBLE_HEIGHT:f32 = 1.;
pub const MAX_INTERACTION_DISTANCE:f32 = 2.;
pub const MAX_INTERACTION_DISTANCE_SQUARED:f32 = MAX_INTERACTION_DISTANCE * MAX_INTERACTION_DISTANCE;
pub const MAX_DRIBBLE_ANGLE:f32 = PI * 0.20;

pub const PLAYER_MAX_CONTROL_DISTANCE:f32 = 0.75;
pub const OPTIMAL_CONTROL_DISTANCE:f32 = 0.75;
pub const SPEED_MATCH_FACTOR:f32 = 20.0;
pub const DISTANCE_MATCH_FACTPR:f32 = 90.0;


#[derive(Debug, Copy, Clone)]
struct ControlCandidate{
	dist_squared:f32,
	entity:Entity, 
	to_control_point:Vec3,
	velocity:Velocity,
}


fn dribble(
	ball:Single<( &mut Velocity, &PhysicalTranslation), With<Ball>>,
	players:Query<(&PhysicalTranslation, &PhysicalRotation, &Velocity, Entity), (With<Player>, Without<Ball>)>,
	mut gizmos: Gizmos,
	time:Res<Time<Fixed>>,
){
	let (mut ball_velocity,  ball_translation) = ball.into_inner();
	if ball_translation.0.y > DRIBBLE_HEIGHT{ return; } //no dribbling high balls!
	let mut closest:Option<ControlCandidate> = None;
	
	//find wgo controls the ball...
	for (translation, rotation, velocity, entity) in players{

		let to_ball = ball_translation.0.xz() - translation.0.xz();
		if to_ball.length_squared() > MAX_INTERACTION_DISTANCE_SQUARED{ continue; }

		let forward = rotation.0 * Vec3::Z;
		let forward_2d = forward.xz();
		let angle_to_ball = to_ball.angle_to(forward_2d);
		//info!("ball angle: {} ", angle_to_ball);
		let target_angle = angle_to_ball.clamp(-MAX_DRIBBLE_ANGLE, MAX_DRIBBLE_ANGLE);
		//info!("ball angle: {}  target angle: {}", angle_to_ball, target_angle);

		//nearest control point
		let control_point = translation.0 + (forward.rotate_y(target_angle) * OPTIMAL_CONTROL_DISTANCE); 
		gizmos.arrow(control_point, ball_translation.0, RED);
		let to_control_point = ball_translation.0 - control_point;

		let dist_squared = to_control_point.length_squared();
		if dist_squared < PLAYER_MAX_CONTROL_DISTANCE * PLAYER_MAX_CONTROL_DISTANCE{
			if closest.is_none() || closest.unwrap().dist_squared > dist_squared{ 
				closest = Some(ControlCandidate { dist_squared, entity, to_control_point, velocity:*velocity });
			}
		}
	}
	if closest.is_none(){
		return;
	}
	if let Some(candidate) = closest{
		let mut ball_vec = ball_velocity.to_vec3();
		let vel_diff = candidate.velocity.to_vec3() - ball_vec;
		let control_force = -candidate.to_control_point * DISTANCE_MATCH_FACTPR;
		let speed_match_force = vel_diff * SPEED_MATCH_FACTOR;
		let combined_force = control_force + speed_match_force;
		ball_vec += time.delta_secs() * combined_force;

		//update ball velocity
		ball_velocity.from_vec3(ball_vec);
		//candidate.to_control_point
		gizmos.arrow(ball_translation.0, ball_translation.0 + vel_diff, BLUE);
		
	};


}


/*
fn decide_influence(
	ball:Single<(&mut Ball, &PhysicalTranslation), Without<Player>>,
	players:Query<(&PhysicalTranslation, &PhysicalRotation, Entity), With<Player>>,
	player_movement:Query<&PlayerMovement>,
){
	let (mut ball, ball_translation) = ball.into_inner();

	let mut candidates:Vec<_> = players.iter().filter_map(|(player_translation, player_rotation, entity)|{

		let diff = player_translation.0.xz() - ball_translation.0.xz();
		let dist_squared = diff.length_squared();
		if dist_squared < PLAYER_MAX_DRIBBLE_DISTANCE * PLAYER_MAX_DRIBBLE_DISTANCE
			&& player_translation.0.y < ball_translation.0.y 
			&& player_translation.0.y + PLAYER_HEIGHT > ball_translation.0.y {
			Some((dist_squared, player_translation.0, player_rotation.0, entity, diff))
		}
		else{
			None
		}
	}).collect::<Vec<(f32, Vec3, Quat, Entity, Vec2)>>();
	//sort by distance
	candidates.sort_by(|p1,p2| p1.0.total_cmp(&p2.0));


	for (len_squared, translation, rotation, entity, diff) in candidates{
		//vertical filter
		let forward_2d = (rotation * Dir3::NEG_Z).xz().normalize_or_zero();
		let dot = diff.dot(forward_2d);
		//let dot = forward_2d.dot(diff);
		if dot < -1.{ continue;} // ball behind the player
		let diff_norm = diff.normalize_or_zero();
		let angle = forward_2d.angle_to(diff_norm).abs();
		
		//within 45 degrees eitherway
		if angle.abs() < PLAYER_DRIBBLE_ANGLE{
			//info!("Control!");
			if let Ok(movement) = player_movement.get(entity){
				let diff_factor =  (PLAYER_OPTIMAL_DRIBBLE_DISTANCE / len_squared.sqrt()).clamp(0.8, 1.2);
				//info!("diff:{}", diff_factor);
				ball.velocity = (movement.velocity() * diff_factor).with_y(ball.velocity.y);
				ball.control = Vec3::ZERO;
				ball.last_touch = Some(entity);
				return;
			}
		}
		else{
			if ball.control == Vec3::ZERO{
				let forward_project = -PLAYER_OPTIMAL_DRIBBLE_DISTANCE * forward_2d;
				let draw_location = Vec3::new(forward_project.x, 0., forward_project.y) + translation;
				ball.control = (draw_location - ball_translation.0).normalize() * 7.0;
				ball.last_touch = Some(entity);
			}
			//info!("draw {}", ball_motion.dribble_draw);
		}
	}
}
 */
fn physics(
	ball:Single<( &mut PhysicalTranslation, &mut Velocity), With<Ball>>,
	time:Res<Time<Fixed>>,
){
	let (mut translation, mut ball_velocity) = ball.into_inner();
	let mut velocity = ball_velocity.to_vec3();
	//ball in the air, apply gravity!
	if translation.0.y > BALL_RADIUS + EPSILON_TOLERANCE{
		//info!("Airborn {} > {}" , translation.0.y, BALL_RADIUS + EPSILON_TOLERANCE);
		let force = AIR_DAMPING * ball_velocity.speed.squared();
		let deceleration = force / BALL_MASS;
		let delta_v = ((-ball_velocity.direction * deceleration) + GRAVITY) * time.delta_secs();
		velocity += delta_v;
	}
	else{
	
		translation.0.y = BALL_RADIUS + EPSILON_TOLERANCE;
		if velocity.y.abs() < 0.1{
			velocity.y = 0.;
		}
		let damping_deceleration = (ball_velocity.direction.xz()) * GROUND_DECELERATION * time.delta_secs(); 
		if damping_deceleration.length_squared() > ball_velocity.speed.squared(){
			velocity = Vec3::ZERO;
		}
		else{
			velocity -= Vec3::ZERO.with_xz(damping_deceleration);
		}
	
	}
	ball_velocity.from_vec3(velocity);
}

fn collisions(
	ball:Single<(&mut PhysicalTranslation, &mut Velocity), With<Ball>>,
	colliders:Query<(Entity, &Collider, &PhysicalTranslation, Option<&Velocity>, &Name), Without<Ball>>,
	time:Res<Time<Fixed>>,
	
	//mut gizmos: Gizmos,
){
	let (mut translation, mut ball_velocity) = ball.into_inner();
	let ball_translation = translation.0;
	let mut ball_movement =  ball_velocity.to_frame_motion(ball_translation, 0., time.delta_secs());
	let mut time_offset = 0.;
	let mut collision_count:usize = 0;
	let mut delta = time.delta_secs();

	while delta > 0. && collision_count < 3 {
		let sphere_sweep = SphereSweep{ 
			start: ball_translation, 
			movement: ball_movement,
			radius: BALL_RADIUS,
		};
		let mut nearest:Option<HitResult> = None;
		let mut target_velocity = Vec3::ZERO;
		let mut restitution = 0.;
		for (entity, collider, translation, velocity, name) in colliders{
			let movement = match velocity{
				Some(velocity) => velocity.to_frame_motion(translation.0, time_offset, delta),
				None => FrameMotion{ origin: translation.0, direction: Dir3::Y, distance: 0. }
			};

			/*
match &collider.shape {
		ColliderShape::Cylinder(cylinder_target) => {

			gizmos.circle(translation.0, cylinder_target.radius, RED);
		},
		ColliderShape::Sphere(sphere_target) => (),
		ColliderShape::Plane(plane_target) => (),
}
 */
			

			if collider.broad_phase(&movement, &sphere_sweep){
				if let Some(hit) = collider.narrow_phase( &movement, entity, &sphere_sweep){

				info!("collision {}", name);
					if nearest.is_none() || hit.time < nearest.unwrap().time{
						nearest = Some(hit);
						restitution = collider.restitution;
						target_velocity = match velocity {
							Some(velocity) => velocity.to_vec3(),
							None =>  Vec3::ZERO,
						}
					};
				};
			}
		}
		if let Some(collision) = nearest{
			collision_count += 1;
			//collision!
			if collision_count > 1{
			info!("Collision! {}", collision_count);
			}
			let time_since_last = delta * collision.time;
			delta -= time_since_last;
			time_offset += time_since_last;
			let collision_point_shifted = collision.point + collision.normal * EPSILON_TOLERANCE;
	
			let ball_v = ball_velocity.to_vec3();
			let approach_v = ball_v - target_velocity;
			let normal_vec:Vec3 = collision.normal.into();
			let approach_speed = approach_v.dot(normal_vec).min(0.);
				
			let next_ball_vel = ball_v - (1.0 + restitution) * approach_speed * normal_vec;
			//info!("bounce! {}", restitution);
			ball_velocity.from_vec3(next_ball_vel);
			ball_movement = ball_velocity.to_frame_motion(collision_point_shifted, 0., delta);
		}
		else{
			translation.0 = ball_movement.final_position();
			delta = 0.;
		}
	}

	ball_velocity.direction = ball_movement.direction;
}




impl Default for Ball{

	fn default()-> Self{
		Self { 
			control:Vec3::ZERO, 
			roll_axis: Dir3::Z, 
			roll_speed: 0., 
			last_touch:None, 
		}
	}
}

