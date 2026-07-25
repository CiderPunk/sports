use std::f32::consts::PI;

use bevy::{color::palettes::css::RED, ecs::error::info, math::{FloatPow, VectorSpace, ops::sqrt}, prelude::*, render::{render_phase::CachedBinKey, render_resource::VertexStepMode}};
use bevy_asset_loader::prelude::*;
use crate::{assets::AssetLoadState, colliders::CollisionCylinder, collisions::{ HitResult, SphereCast}, game_gizmos::GizmoSpawnMessage, game_schedule::GameSchedule, game_state::GameState, player::{InfluenceZone, Movement, PLAYER_DRIBBLE_ANGLE, PLAYER_HEIGHT, PLAYER_MAX_DRIBBLE_DISTANCE, PLAYER_OPTIMAL_DRIBBLE_DISTANCE, Player}};


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
			.add_systems(Update, (decide_influence, update_ball).chain().in_set(GameSchedule::MoveBall))
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

fn spawn_ball(
	mut commands:Commands,
	ball_assets:Res<BallAssets>,
){
	commands.spawn((
		Ball,
		WorldAssetRoot(ball_assets.ball_scene.clone()),
		//Transform::from_translation(Vec3::new(0., BALL_GROUND_LEVEL ,0.)).with_scale(Vec3::splat(BALL_SCALE)),
		Transform::from_translation(Vec3::new(-30., 10. ,0.)).with_scale(Vec3::splat(BALL_SCALE)),
		BallMotion{
			direction: Dir3::X,
			speed: 5.0,
			..default()
		}
	));
}

fn separate_inclusions(
	transform:&mut Transform,
	candidates:&Vec<(&Transform, &CollisionCylinder, Entity)>,
){
	for _ in 0 .. 3{
		let sphere_cast = SphereCast{ origin: transform.translation, direction: Dir3::X, radius: BALL_RADIUS, distance: 0. };
		let mut moved = false;
		for ((player_transform, player_collision, _)) in candidates.iter(){
			if let Some(inclusion_result) = sphere_cast.inclusion_vertical_cylinder(player_transform.translation, player_collision.radius, player_collision.height){
				transform.translation += inclusion_result.correction;
				moved = true;
			};
		}
		if !moved { break; }
	}
}

fn get_next_collision(
	transform:&Transform,
	direction:&Dir3,
	distance:f32,
	candidates:&Vec<(&Transform, &CollisionCylinder, Entity)>,
)->Option<HitResult>{
	let sphere_cast = SphereCast{ origin: transform.translation, direction:*direction,  radius: BALL_RADIUS, distance: distance };
	let mut closest:Option<HitResult> = None;
	for (player_transform, collision_cylinder, entity) in candidates.iter(){
		if let Some(hit) = sphere_cast.intersect_vertical_cylinder(player_transform.translation, collision_cylinder.radius, collision_cylinder.height, *entity){
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

#[derive(Copy, Clone)]
enum Zone{
	static_zone,
	control_zone,
}


#[derive(Copy, Clone)]
struct InfluencerCandidate{
	zone:Zone,
	zones:InfluenceZone,
	dist_squared:f32,
	entity:Entity,
	origin:Vec3,
}

const MAX_INFLUENCE:f32 = 2.0;


fn decide_influence(
	ball:Single<(&mut BallMotion, &Transform), Without<Player>>,
	players:Query<(&GlobalTransform, Entity), With<Player>>,
	player_movement:Query<&Movement>,
){
	let (mut ball_motion, ball_transform) = ball.into_inner();

let mut candidates:Vec<_> = players.iter().filter_map(|(transform, entity)|{
	let player_translation = transform.translation();
		let diff = player_translation.xz() - ball_transform.translation.xz();
		let dist_squared = diff.length_squared();
		if dist_squared < PLAYER_MAX_DRIBBLE_DISTANCE * PLAYER_MAX_DRIBBLE_DISTANCE
			&& player_translation.y < ball_transform.translation.y 
			&& player_translation.y + PLAYER_HEIGHT > ball_transform.translation.y {
			Some((dist_squared, transform, entity, diff))
		}
		else{
			None
		}
	}).collect::<Vec<(f32, &GlobalTransform, Entity, Vec2)>>();
	//sort by distance
	candidates.sort_by(|p1,p2| p1.0.total_cmp(&p2.0));


	for (len_squared, transform, entity, diff) in candidates{
		//vertical filter
		let forward_2d = transform.forward().xz().normalize_or_zero();
		let dot = diff.dot(forward_2d);
		//let dot = forward_2d.dot(diff);
		if dot < 0.{ continue;} // ball behind the player
		let diff_norm = diff.normalize_or_zero();
		let angle = forward_2d.angle_to(diff_norm).abs();
		
		//within 45 degrees eitherway
		if angle.abs() < PLAYER_DRIBBLE_ANGLE{
			//info!("Control!");
			if let Ok(movement) = player_movement.get(entity){
				let diff_factor =  (PLAYER_OPTIMAL_DRIBBLE_DISTANCE / len_squared.sqrt()).clamp(0.8, 1.2);
				//info!("diff:{}", diff_factor);
				let velocity = (movement.velocity() * diff_factor).with_y(ball_motion.direction.y * ball_motion.speed);
				if let Ok((direction, speed)) = Dir3::new_and_length(velocity){
					ball_motion.direction = direction;
					ball_motion.speed = speed;
					ball_motion.dribble_draw = Vec3::ZERO;
					ball_motion.last_touch = Some(entity);
				};
				return;
			}
		}
		else{
			if ball_motion.dribble_draw == Vec3::ZERO{
				let forward_project = -PLAYER_OPTIMAL_DRIBBLE_DISTANCE * forward_2d;
				let draw_location = Vec3::new(forward_project.x, 0., forward_project.y) + transform.translation();
				ball_motion.dribble_draw = (draw_location - ball_transform.translation).normalize() * 4.0;
				ball_motion.last_touch = Some(entity);
			}
			//info!("draw {}", ball_motion.dribble_draw);
		}
	}
}


/*
fn decide_influence(
	ball:Single<(&mut BallMotion, &Transform), Without<Player>>,
	influencers:Query<(&GlobalTransform,&InfluenceZone, &ChildOf)>,
	player_movement_query:Query<&Movement>,
	time:Res<Time>,
){
	let (mut motion, ball_transform) = ball.into_inner();
	let hits:Vec<InfluencerCandidate> = influencers.iter().filter_map(|(transform, influence, child_of)| { 
		let translation = transform.translation();
		let dist_squared = (translation - ball_transform.translation).length_squared();
		if dist_squared < influence.static_radius.squared(){
			Some(InfluencerCandidate { zone: Zone::static_zone, zones:*influence, dist_squared, entity: child_of.0, origin: translation })
		}
		else if dist_squared < influence.draw_radius.squared(){
			Some(InfluencerCandidate { zone: Zone::control_zone, zones:*influence, dist_squared, entity:child_of.0, origin: translation })
		}
		else{
			None
		}
	}).collect();	

	//TODO: this should consider all players with the ball in their influence zone
	let mut closest = hits.first();
	for hit in hits.iter(){
		if hit.dist_squared < closest.unwrap().dist_squared { 
			closest = Some(hit); 
		}
	}

	if let Some(closest) = closest && let Ok(player_movement) = player_movement_query.get(closest.entity){
		let velocity = player_movement.velocity();
		let control_velocity = match closest.zone{
			Zone::control_zone =>{ 
				let distance = closest.dist_squared.sqrt() - closest.zones.static_radius;
				(closest.origin - ball_transform.translation).normalize_or(Vec3::ZERO) * distance * 2.0
			},
			Zone::static_zone => Vec3::ZERO,
		};

		motion.dribble_draw = motion.dribble_draw.lerp(control_velocity, time.delta_secs() * 2.0);
		if let Ok((direction, speed)) = Dir3::new_and_length(velocity){
			motion.direction = direction;
			motion.speed = speed;
		};
	
	};
}
	 */


fn update_ball(
	ball:Single<(&mut BallMotion, &mut Transform), Without<Player>>,
	players:Query<(&Transform,&CollisionCylinder, Entity), With<Player>>,
	time:Res<Time>,
	mut gizmos: Gizmos,
	//mut gizmo_writer:MessageWriter<GizmoSpawnMessage>,
){
	let (mut motion, mut transform) = ball.into_inner();
	let speed = motion.speed;
	let mut distance = speed * time.delta_secs();
	let mut direction =  motion.direction;

	let sphere_cast = SphereCast{ origin: transform.translation, direction, radius: BALL_RADIUS, distance };
	
	gizmos.arrow(transform.translation, transform.translation + motion.dribble_draw, RED);
	transform.translation += motion.dribble_draw * time.delta_secs();
	motion.dribble_draw = Vec3::ZERO;
	//motion.dribble_draw = motion.dribble_draw.lerp(Vec3::ZERO, time.delta_secs() * 4.0);


	//broad filter for local collision candidates
	let candidates:Vec<_> = players.iter().filter(|(player_transform, collision, entity)| 
		motion.last_touch.as_ref() != Some(entity)
		&& sphere_cast.cylinder_candidate_filter(player_transform.translation, collision.radius) 
	).collect();

	separate_inclusions(&mut transform, &candidates);

	//collision detection
	if speed > f32::EPSILON{
		while distance > 0.{
			let hit = get_next_collision(&transform, &direction, distance, &candidates);
			if let Some(hit) = hit{
				//debugging
				//info!("Player hit {} player position: {} hit position: {}", hit.entity, hit.other_origion, hit.position);
				//info!("Sphere cast; origin: {}, direction:{}, radius:{}, distance:{}", sphere_cast.origin, sphere_cast.direction, sphere_cast.radius, sphere_cast.distance);
				//gizmo_writer.write(GizmoSpawnMessage::new(transform.clone(), crate::game_gizmos::GizmoColour::White));
				//gizmo_writer.write(GizmoSpawnMessage::new(Transform::from_translation(hit.position), crate::game_gizmos::GizmoColour::Pink));
				//end debugging

				info!("Collision: {} {:?}", hit.entity, motion.last_touch);
				transform.translation = hit.position;		
				distance -= hit.distance;
				direction = Dir3::new_unchecked( direction.reflect(*hit.normal));
			}
			else{
				transform.translation += *direction * distance;
				distance = 0.;
			}
			motion.direction = direction;
		}
	}

	//update velocity
	let mut velocity = speed * direction;

	//ball in the air, apply gravity!
	if transform.translation.y > BALL_RADIUS{
		let force = AIR_DAMPING * speed * speed;
		let deceleration = force / BALL_MASS;
		let delta_v = ((-direction * deceleration) + (-Vec3::Y * GRAVITY)) * time.delta_secs();
		velocity += delta_v;
	}
	else{
		//TODO: touched ground - update roll
		motion.roll_axis = Dir3::new(direction.cross(Vec3::Y)).unwrap_or(Dir3::Y);
		motion.roll_speed = motion.speed * PI * BALL_RADIUS;

		if velocity.y < -MIN_BOUNCE_SPEED {
			//bounce!
			//info!("bounce {}", velocity.y);
			velocity.y *= -BALL_COEFFECIENT_OF_RESTITUTION;
		}			
		else{
			let damping_deceleration = (direction.xz()) * GROUND_DECELERATION * time.delta_secs(); 
			if damping_deceleration.length_squared() > velocity.xz().length_squared(){	
				velocity = Vec3::ZERO;
			}
			else{
				//info!("ground deceleration {}", damping_deceleration);
				velocity -= Vec3::new(damping_deceleration.x, 0., damping_deceleration.y);
			}
		}		
		transform.translation.y = BALL_RADIUS;
	}
	//info!("ball velocity:{}", velocity);
	transform.rotate_axis(motion.roll_axis, -motion.roll_speed * time.delta_secs());
	//motion.direction = Dir3::new_unchecked(velocity.normalize_or(Vec3::Y));
	motion.direction = Dir3::new(velocity).unwrap_or(Dir3::Y);
	motion.speed = velocity.length();
}



#[derive(Component, Debug)]
#[require(BallMotion)]
pub struct Ball;

#[derive(Component, Debug)]
pub struct BallMotion{
	//pub velocity:Vec3,
	pub direction:Dir3,
	pub speed:f32,
	dribble_draw:Vec3,
	roll_axis:Dir3,
	roll_speed:f32,
	last_touch:Option<Entity>,
}

impl Default for BallMotion{

	fn default()-> Self{
		Self { direction: Dir3::Y, dribble_draw:Vec3::ZERO, speed: 0., roll_axis: Dir3::Z, roll_speed: 0., last_touch:None, }
	}
}

