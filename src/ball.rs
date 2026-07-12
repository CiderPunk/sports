use std::f32::consts::PI;

use bevy::{math::VectorSpace, prelude::*};

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
			.add_systems(Update, move_ball.in_set(GameSchedule::MoveBall))
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
			velocity: Vec3 { x: 5., y: 0., z: 0. },
			..default()
		}
	));
}

fn move_ball(
	ball:Single<(&mut BallMotion, &mut Transform), Without<Player>>,
	players:Query<(&Transform,&CollisionCylinder, Entity), With<Player>>,
	player_velocity:Query<&Movement>,
	time:Res<Time>,
	mut gizmo_writer:MessageWriter<GizmoSpawnMessage>,
){
	let (mut motion, mut transform) = ball.into_inner();
	let speed = motion.velocity.length();
	if speed >f32::EPSILON {
		let sphere_cast = SphereCast{ origin: transform.translation, direction: Dir3::new_unchecked( motion.velocity / speed), radius: BALL_RADIUS, distance: speed * time.delta_secs() };

		let mut closest:Option<HitResult> = None;
		let mut player_position:Option<Vec3> = None;
		for (player_transform, collision_cylinder, entity) in players{
			if let Some(hit) = sphere_cast.interset_vertical_cylinder(player_transform.translation, collision_cylinder.radius, collision_cylinder.height, entity){
				match closest {
					Some(last) => if hit.distance < last.distance { 
						closest = Some(hit);
						player_position = Some(player_transform.translation);
					}
					None=> {
						closest =Some(hit);
						player_position = Some(player_transform.translation);
					}
				}
			}
		}

		if let Some(hit) = closest{
			let player_velocity = if let Ok(player_movement) = player_velocity.get(hit.entity){
				player_movement.velocity()
			}else{ 
				Vec3::ZERO 
			};
			info!("Player hit {} player position: {} hit position: {}", hit.entity, player_position.unwrap(), hit.position);
			info!("Sphere cast; origin: {}, direction:{}, radius:{}, distance:{}", sphere_cast.origin, sphere_cast.direction, sphere_cast.radius, sphere_cast.distance);
			
			gizmo_writer.write(GizmoSpawnMessage::new(transform.clone(), crate::game_gizmos::GizmoColour::White));
			gizmo_writer.write(GizmoSpawnMessage::new(Transform::from_translation(hit.position), crate::game_gizmos::GizmoColour::Pink));

			transform.translation = hit.position;
			motion.velocity =  motion.velocity.reflect(*hit.normal);// + player_velocity;
		}
		else{
			transform.translation += motion.velocity * time.delta_secs();
		}

		

		//info!("ball translation:{} velocity:{}", transform.translation, motion.velocity);
		//rotate it!
		if let Some(axis) = motion.roll_axis{
			transform.rotate_axis(axis, -motion.roll_speed * time.delta_secs());
		}
	}

	//ball in the air, apply gravity!
	if transform.translation.y > BALL_RADIUS{
		let force = AIR_DAMPING * motion.velocity.length_squared();
		let deceleration = force / BALL_MASS;
		let delta_v = ((-motion.velocity.normalize() * deceleration) + (-Vec3::Y * GRAVITY)) * time.delta_secs();
		motion.velocity += delta_v;
	}
	else{
		//roll speed

		if motion.velocity.z > f32::EPSILON || motion.velocity.x > f32::EPSILON{
			motion.roll_axis = Dir3::from_xyz(motion.velocity.z, 0., motion.velocity.x).ok();
			motion.roll_speed = motion.velocity.xz().length() * PI * BALL_RADIUS;
		}
		else{
			motion.roll_axis = None;
		}

		if motion.velocity.y < -MIN_BOUNCE_SPEED {
			//bounce!
			info!("bounce {}", motion.velocity.y);
			motion.velocity.y *= -BALL_COEFFECIENT_OF_RESTITUTION;
		}			
		else{
			let damping_deceleration = (motion.velocity.normalize().xz()) * GROUND_DECELERATION * time.delta_secs(); 
			if damping_deceleration.length_squared() > motion.velocity.xz().length_squared(){	
				motion.velocity = Vec3::ZERO;
			}
			else{
				//info!("ground deceleration {}", damping_deceleration);
				motion.velocity -= Vec3::new(damping_deceleration.x, 0., damping_deceleration.y);
			}
		}		
		transform.translation.y = BALL_RADIUS;
	} 
}



#[derive(Component)]
#[require(BallMotion)]
pub struct Ball;

#[derive(Component, Default)]
pub struct BallMotion{
	pub velocity:Vec3,
	roll_axis:Option<Dir3>,
	roll_speed:f32,
}



