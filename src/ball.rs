use std::{f32::consts::PI, ops::Mul};

use bevy::{color::palettes::css::RED, log::tracing_subscriber::fmt::time, math::{FloatPow, VectorSpace}, prelude::*};
use bevy_asset_loader::prelude::*;
use crate::{assets::AssetLoadState, game_schedule::GameSchedule, game_state::GameState, interpolation::{PhysicalRotation, PhysicalTranslation}, physics::{Collidable, Collider, ColliderShape, EPSILON_TOLERANCE, FrameMotion, HitResult, PhysicalProperties, SphereSweep, SphereTarget, Velocity}, player::{ PLAYER_DRIBBLE_ANGLE, PLAYER_HEIGHT, PLAYER_MAX_DRIBBLE_DISTANCE, PLAYER_OPTIMAL_DRIBBLE_DISTANCE, Player, PlayerMovement}};


const BALL_SCALE: f32 = 0.5;
pub const BALL_RADIUS:f32 = 0.25 * BALL_SCALE;
const GRAVITY:f32 = 9.8;
const BALL_COEFFECIENT_OF_RESTITUTION:f32 = 0.75;
const MIN_BOUNCE_SPEED:f32 = 0.8;

//air damping
const DRAG_COEFFICIENT:f32 = 0.30;
const AIR_DENSITY:f32 = 1.225;
const BALL_CROSS_SECTION_AREA:f32 = 0.038;
const AIR_DAMPING:f32 = 0.5 * AIR_DENSITY * BALL_CROSS_SECTION_AREA * DRAG_COEFFICIENT;

//ground damping
//const ROLLING_RESISTANCE:f32 = 0.08;
const ROLLING_RESISTANCE:f32 = 0.2;
const BALL_MASS:f32 = 0.43;
//don't need ball mass!
//const GROUND_DECELERATION:f32 = (BALL_MASS * GRAVITY * ROLLING_RESISTANCE) / BALL_MASS;  
const GROUND_DECELERATION:f32 = GRAVITY * ROLLING_RESISTANCE;  


pub struct BallPlugin;
impl Plugin for BallPlugin{
	fn build(&self, app: &mut App) {
		app
			.configure_loading_state(
				LoadingStateConfig::new(AssetLoadState::Startup)
				.load_collection::<BallAssets>(),
			)
			.add_systems(OnEnter(GameState::Playing), spawn_ball)
			.add_systems(FixedUpdate, physics.in_set(GameSchedule::PreMovement))
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
		},
		PhysicalProperties{ restitution: 0.95, mass: 1.0 },
		Velocity{ direction: Dir3::X, speed:5. },
		PhysicalTranslation(Vec3::new(-30., 10. ,0.)),
	));
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
	ball:Single<( &PhysicalTranslation, &mut Velocity, &PhysicalProperties), With<Ball>>,
	time:Res<Time<Fixed>>,
){
	let (translation, mut ball_velocity, ball_props) = ball.into_inner();
	let mut velocity = ball_velocity.to_vec3();
	//ball in the air, apply gravity!
	if translation.0.y > BALL_RADIUS{
		let force = AIR_DAMPING * ball_velocity.speed.squared();
		let deceleration = force / BALL_MASS;
		let delta_v = ((-ball_velocity.direction * deceleration) + (-Vec3::Y * GRAVITY)) * time.delta_secs();
		velocity += delta_v;
	}
	else{
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
	colliders:Query<(Entity, &Collider, &PhysicalTranslation, Option<&Velocity>), Without<Ball>>,
	collision_target_query:Query<(&Velocity, &PhysicalProperties), Without<Ball>>,
	time:Res<Time<Fixed>>,
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
		for (entity, collider, translation, velocity) in colliders{
			let movement = match velocity{
				Some(velocity) => velocity.to_frame_motion(translation.0, time_offset, delta),
				None => FrameMotion{ origin: translation.0, direction: Dir3::Y, distance: 0. }
			};
			if collider.broad_phase(&movement, &sphere_sweep){
				if let Some(hit) = collider.narrow_phase( &movement, entity, &sphere_sweep){
					if let Some(ref near) = nearest{
						if hit.time < near.time{
							nearest = Some(hit);
						}
					}
					else{
						nearest = Some(hit);
					}
				};
			}
		}
		if let Some(collision) = nearest{
			collision_count += 1;
			//collision!
			info!("Collision! {}", ball_velocity.direction);
			let time_since_last = delta * collision.time;
			delta -= time_since_last;
			time_offset += time_since_last;
			let collision_point_shifted = collision.point + collision.normal * EPSILON_TOLERANCE;
			if let Ok((target_velocity, target_props)) = collision_target_query.get(collision.entity){
				
				let target_v = target_velocity.to_vec3();
				let ball_v = ball_velocity.to_vec3();
				let approach_v = ball_v - target_v;
				let normal_vec:Vec3 = collision.normal.into();
				let approach_speed = approach_v.dot(normal_vec);
				let next_ball_vel = ball_v - (1.0 + target_props.restitution) * approach_speed * normal_vec;
				
				ball_velocity.from_vec3(next_ball_vel);
				ball_movement = ball_velocity.to_frame_motion(collision_point_shifted, 0., delta);
			}
			else{
				ball_movement.distance = (1.0 - collision.time) * ball_movement.distance;
				ball_movement.origin = collision_point_shifted;
				ball_movement.direction = Dir3::new(ball_movement.direction.reflect(collision.normal.into())).unwrap_or(Dir3::Y);
			}

		}
		else{
			translation.0 = ball_movement.final_position();
			delta = 0.;
		}
	}

	ball_velocity.direction = ball_movement.direction;
}



/*
fn update_ball(
	ball:Single<(&mut Ball, &mut PhysicalTranslation), Without<Player>>,
	players:Query<(&PhysicalTranslation, &CollisionCylinder, Entity), With<Player>>,
	time:Res<Time<Fixed>>,
	mut gizmos: Gizmos,
	//mut gizmo_writer:MessageWriter<GizmoSpawnMessage>,
){
	let (mut ball, mut translation) = ball.into_inner();
	let mut direction = Dir3::new(ball.velocity).unwrap_or(-Dir3::Y);
	let speed = ball.velocity.length();
	let mut distance = speed * time.delta_secs();
	

	let sphere_cast = SphereCast{ origin: translation.0, direction, radius: BALL_RADIUS, distance };
	
	gizmos.arrow(translation.0, translation.0 + ball.control, RED);
	translation.0 += ball.control * time.delta_secs();
	ball.control = Vec3::ZERO;

	//broad filter for local collision candidates
	let candidates:Vec<_> = players.iter().filter(|(player_translation, collision, entity)| 
		ball.last_touch.as_ref() != Some(entity)
		&& sphere_cast.cylinder_candidate_filter(player_translation.0, collision.radius) 
	).collect();

	separate_inclusions(&mut translation.0, &candidates);

	//collision detection

	while distance > 0.{
		let hit = get_next_collision(translation.0, &direction, distance, &candidates);
		if let Some(hit) = hit{
			info!("Collision: {} {:?}", hit.entity, ball.last_touch);
			translation.0 = hit.position;		
			distance -= hit.distance;
			direction = Dir3::new_unchecked( direction.reflect(*hit.normal));
		}
		else{
			translation.0 += *direction * distance;
			distance = 0.;
		}
		//ball.velocity = direction * speed ;
	}

	//ball in the air, apply gravity!
	if translation.0.y > BALL_RADIUS{
		let force = AIR_DAMPING * speed * speed;
		let deceleration = force / BALL_MASS;
		let delta_v = ((-direction * deceleration) + (-Vec3::Y * GRAVITY)) * time.delta_secs();
		ball.velocity += delta_v;
	}
	else{
		//TODO: touched ground - update roll
		ball.roll_axis = Dir3::new(direction.cross(Vec3::Y)).unwrap_or(Dir3::Y);
		ball.roll_speed = speed * PI * BALL_RADIUS;

		if ball.velocity.y < -MIN_BOUNCE_SPEED {
			//bounce!
			//info!("bounce {}", velocity.y);
			ball.velocity.y *= -BALL_COEFFECIENT_OF_RESTITUTION;
		}			
		else{
			let damping_deceleration = (direction.xz()) * GROUND_DECELERATION * time.delta_secs(); 
			if damping_deceleration.length_squared() > ball.velocity.xz().length_squared(){	
				ball.velocity = Vec3::ZERO;
			}
			else{
				//info!("ground deceleration {}", damping_deceleration);
				ball.velocity -= Vec3::new(damping_deceleration.x, 0., damping_deceleration.y);
			}
		}		
		translation.0.y = BALL_RADIUS;
	}
	//info!("ball velocity:{}", velocity);
	//transform.rotate_axis(ball.roll_axis, -ball.roll_speed * time.delta_secs());
	//motion.direction = Dir3::new_unchecked(velocity.normalize_or(Vec3::Y));
}

 */


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

