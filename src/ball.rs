use std::{f32::consts::PI, ops::Mul};

use bevy::{color::palettes::css::RED, prelude::*};
use bevy_asset_loader::prelude::*;
use crate::{assets::AssetLoadState, colliders::CollisionCylinder, collisions::{ HitResult, SphereCast}, game_schedule::GameSchedule, game_state::GameState, interpolation::{PhysicalRotation, PhysicalTranslation}, player::{ self, Movement, PLAYER_DRIBBLE_ANGLE, PLAYER_HEIGHT, PLAYER_MAX_DRIBBLE_DISTANCE, PLAYER_OPTIMAL_DRIBBLE_DISTANCE, Player}};


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
			.add_systems(FixedUpdate, (decide_influence, update_ball).chain().in_set(GameSchedule::MoveBall))
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
	pub velocity:Vec3,
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
			velocity: Dir3::X * 5.,
			..default()
		},
		PhysicalTranslation(Vec3::new(-30., 10. ,0.)),
	));
}

fn separate_inclusions(
	translation:&mut Vec3,
	candidates:&Vec<(&PhysicalTranslation, &CollisionCylinder, Entity)>,
){
	for _ in 0 .. 3{
		let sphere_cast = SphereCast{ origin: *translation, direction: Dir3::X, radius: BALL_RADIUS, distance: 0. };
		let mut moved = false;
		for (player_transform, player_collision, _) in candidates.iter(){
			if let Some(inclusion_result) = sphere_cast.inclusion_vertical_cylinder(player_transform.0, player_collision.radius, player_collision.height){
				*translation += inclusion_result.correction;
				moved = true;
			};
		}
		if !moved { break; }
	}
}

fn get_next_collision(
	translation:Vec3,
	direction:&Dir3,
	distance:f32,
	candidates:&Vec<(&PhysicalTranslation, &CollisionCylinder, Entity)>,
)->Option<HitResult>{
	let sphere_cast = SphereCast{ origin:translation, direction:*direction,  radius: BALL_RADIUS, distance: distance };
	let mut closest:Option<HitResult> = None;
	for (translation, collision_cylinder, entity) in candidates.iter(){
		if let Some(hit) = sphere_cast.intersect_vertical_cylinder(translation.0, collision_cylinder.radius, collision_cylinder.height, *entity){
			match closest {
				Some(last) => if hit.distance < last.distance { 
					closest = Some(hit);
				}
				None=> {
					closest =Some(hit);
				}
			}
		}
	}
	closest
}

fn decide_influence(
	ball:Single<(&mut Ball, &PhysicalTranslation), Without<Player>>,
	players:Query<(&PhysicalTranslation, &PhysicalRotation, Entity), With<Player>>,
	player_movement:Query<&Movement>,
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




impl Default for Ball{

	fn default()-> Self{
		Self { 
			velocity: Vec3::Y,
			control:Vec3::ZERO, 
			roll_axis: Dir3::Z, 
			roll_speed: 0., 
			last_touch:None, 
		}
	}
}

