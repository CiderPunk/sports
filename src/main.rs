mod player;
mod assets;
mod macros;
mod game_state;
mod ball;
mod pitch;
use std::f32::consts::PI;

use bevy::{color::palettes::css::WHITE, core_pipeline::tonemapping::Tonemapping, light::{CascadeShadowConfig, CascadeShadowConfigBuilder, DirectionalLightShadowMap}, prelude::*};
use crate::{assets::AssetsPlugin, ball::BallPlugin, game_state::GameStatePlugin, pitch::PitchPlugin, player::PlayerPlugin};

const APP_NAME: &str = "Sportsball";
fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(WindowPlugin{
			primary_window: Some(Window{
				title: APP_NAME.into(),
				name: Some(APP_NAME.into()),
				fit_canvas_to_parent: true,
				visible:true,
				..default()
			}),
			..default()
		}))
		.add_plugins((
			GameStatePlugin,
			AssetsPlugin,
			PlayerPlugin,
			BallPlugin,
			PitchPlugin,

		))
		.insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
    .insert_resource(GlobalAmbientLight {
        color: WHITE.into(),
        brightness: 1_000.0,
        ..default()
    })
		.insert_resource(DirectionalLightShadowMap { size: 4096 })
		.add_systems(Startup, init_camera)
		.run();
}



fn init_camera(mut commands:Commands){
	// Narrow FOV (~12 degrees) simulates a long telephoto lens
	let long_lens_fov = 12.0 * PI / 180.0; 
  commands.spawn((
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
		//Transform::from_translation(Vec3::new(0.,20.,12.)).looking_at(Vec3::ZERO, Vec3::Y),
		//Transform::from_translation(Vec3::new(0.,100.,64.)).looking_at(Vec3::ZERO, Vec3::Y),
		
		//best!
		Transform::from_translation(Vec3::new(0.,140.,120.)).looking_at(Vec3::ZERO, Vec3::Y),
		//Transform::from_translation(Vec3::new(0.,320.,220.)).looking_at(Vec3::ZERO, Vec3::Y),
  ));
	commands.spawn((
		DirectionalLight {
			color: WHITE.into(),
			shadow_maps_enabled:true,
			illuminance:5_000.,
			contact_shadows_enabled:true,
			shadow_depth_bias:0.2,
			shadow_normal_bias:0.2,
				..default()
		},
		CascadeShadowConfigBuilder {
				maximum_distance: 500.0, // Adjust this higher until shadows stop clipping
				..default()
		}.build(),
		Transform::from_xyz(300., 1000., 200.).looking_at(Vec3::ZERO, Vec3::Y),
	));
}
