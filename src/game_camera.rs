use std::f32::consts::PI;

use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*};

use crate::ball::Ball;


//const CAMERA_OFFSET:Vec3 = Vec3::new(0.,140.,120.);
const CAMERA_OFFSET:Vec3 = Vec3::new(0.,80.,60.);

pub struct GameCameraPlugin;
impl Plugin for GameCameraPlugin{
	fn build(&self, app: &mut App) {
		app
			.add_systems(Startup, spawn_camera)
			.add_systems(Update, track_ball)
			;
	}
}

#[derive(Component)]
pub struct BallCamera;


fn spawn_camera(
	mut commands:Commands
){
	// Narrow FOV (~12 degrees) simulates a long telephoto lens
	let long_lens_fov = 12.0 * PI / 180.0; 
  commands.spawn((
		BallCamera,
    Camera3d{
		
      ..default()
    },
    Camera {
      order: 1, 
      ..default()
    },
		Projection::Perspective(PerspectiveProjection { 
			fov: long_lens_fov,
			..default()
		}),
    Tonemapping::BlenderFilmic,
		//best!
		Transform::from_translation(CAMERA_OFFSET).looking_at(Vec3::ZERO, Vec3::Y),
	));
}

fn track_ball(
	ball:Single<&GlobalTransform, With<Ball>>,
	mut camera:Single<&mut Transform, With<BallCamera>>
){
	camera.translation = Vec3::new(ball.translation().x, 0., ball.translation().z) + CAMERA_OFFSET;
}



