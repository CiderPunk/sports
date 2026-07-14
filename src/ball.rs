use std::f32::consts::PI;

use bevy::{log::tracing_subscriber::filter::filter_fn, math::VectorSpace, prelude::*};

use bevy_asset_loader::prelude::*;
use crate::{assets::AssetLoadState, colliders::CollisionCylinder, collisions::{ HitResult, SphereCast}, game_gizmos::GizmoSpawnMessage, game_schedule::GameSchedule, game_state::GameState, player::{Movement, Player}};


const BALL_SCALE: f32 = 0.5;
const BALL_RADIUS:f32 = 0.25 * BALL_SCALE;
const GRAVITY:f32 = 9.8;
const BALL_COEFFECIENT_OF_RESTITUTION:f32 = 0.75;
const MIN_BOUNCE_SPEED:f32 = 0.8;

//air damping
const DRAG_COEFFICIENT:f32 = 0.30;
const AIR_DENSITY:f32 = 1.225;
const BALL_CROSS_SECTION_AREA:f32 = 0.038;
const AIR_DAMPING:f32 = 0.5 * AIR_DENSITY * BALL_CROSS_SECTION_AREA * DRAG_COEFFICIENT;

//ground damping
const ROLLING_RESISTANCE:f32 = 0.08;
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
			.add_systems(Update, update_ball.in_set(GameSchedule::MoveBall))
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

fn separate_inlcusions(
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
	let mut player_position:Option<Vec3> = None;
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



fn update_ball(
	ball:Single<(&mut BallMotion, &mut Transform), Without<Player>>,
	players:Query<(&Transform,&CollisionCylinder, Entity), With<Player>>,
	player_velocity:Query<&Movement>,
	time:Res<Time>,
	mut gizmo_writer:MessageWriter<GizmoSpawnMessage>,
){
	let (mut motion, mut transform) = ball.into_inner();
	let speed = motion.speed;
	let mut distance = speed * time.delta_secs();
	let mut direction =  motion.direction;

	let sphere_cast = SphereCast{ origin: transform.translation, direction, radius: BALL_RADIUS, distance };

	//broad filter for local collision candidates
	let candidates:Vec<_> = players.iter().filter(|(player_transform, collision, _entity)|  sphere_cast.cylinder_candidate_filter(player_transform.translation, collision.radius) ).collect();

	separate_inlcusions(&mut transform, &candidates);

	if speed > f32::EPSILON{
		while distance > 0.{

			let hit = get_next_collision(&transform, &direction, distance, &candidates);

			if let Some(hit) = hit{
				let player_velocity = if let Ok(player_movement) = player_velocity.get(hit.entity){
					player_movement.velocity()
				}else{ 
					Vec3::ZERO 
				};

				//debugging
				//info!("Player hit {} player position: {} hit position: {}", hit.entity, hit.other_origion, hit.position);
				//info!("Sphere cast; origin: {}, direction:{}, radius:{}, distance:{}", sphere_cast.origin, sphere_cast.direction, sphere_cast.radius, sphere_cast.distance);
				//gizmo_writer.write(GizmoSpawnMessage::new(transform.clone(), crate::game_gizmos::GizmoColour::White));
				//gizmo_writer.write(GizmoSpawnMessage::new(Transform::from_translation(hit.position), crate::game_gizmos::GizmoColour::Pink));
				//end debugging

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

	let mut velocity = speed * direction;

	//ball in the air, apply gravity!
	if transform.translation.y > BALL_RADIUS{
		let force = AIR_DAMPING * speed * speed;
		let deceleration = force / BALL_MASS;
		let delta_v = ((-direction * deceleration) + (-Vec3::Y * GRAVITY)) * time.delta_secs();
		velocity += delta_v;
	}
	else{
		//touched ground - update roll
		motion.roll_axis = Dir3::new_unchecked(direction.cross(Vec3::Y).normalize());
		motion.roll_speed = motion.speed * PI * BALL_RADIUS;

		if velocity.y < -MIN_BOUNCE_SPEED {
			//bounce!
			info!("bounce {}", velocity.y);
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
	transform.rotate_axis(motion.roll_axis, -motion.roll_speed * time.delta_secs());
	motion.direction = Dir3::new_unchecked(velocity.normalize_or(Vec3::Y));
	motion.speed = velocity.length();

}



#[derive(Component)]
#[require(BallMotion)]
pub struct Ball;

#[derive(Component)]
pub struct BallMotion{
	//pub velocity:Vec3,
	pub direction:Dir3,
	pub speed:f32,
	roll_axis:Dir3,
	roll_speed:f32,
}

impl Default for BallMotion{

	fn default()-> Self{
		Self { direction: Dir3::Y, speed: 0., roll_axis: Dir3::Z, roll_speed: 0.}
	}
}

