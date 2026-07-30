mod player;
mod assets;
mod macros;
mod game_state;
mod ball;
mod pitch;
mod game_control;
mod game_schedule;
mod game_camera;
mod collisions;
mod colliders;
mod tests;
mod game_gizmos;
mod kit;
mod flag;
mod animation_manager;
use std::f32::consts::PI;

use bevy::{color::palettes::css::WHITE, core_pipeline::tonemapping::Tonemapping, light::{ CascadeShadowConfigBuilder, DirectionalLightShadowMap}, prelude::*};
use bevy_enhanced_input::EnhancedInputPlugin;
use bevy_prng::WyRand;
use bevy_rand::plugin::EntropyPlugin;
use crate::{animation_manager::AnimatedMeshPlugin, assets::AssetsPlugin, ball::BallPlugin, flag::FlagPlugin, game_camera::GameCameraPlugin, game_control::GameControlPlugin, game_gizmos::GameGizmosPlugin, game_schedule::GameSchedulePlugin, game_state::GameStatePlugin, kit::KitPlugin, pitch::PitchPlugin, player::PlayerPlugin};

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
			EnhancedInputPlugin,
			EntropyPlugin::<WyRand>::default()

		))		
		.add_plugins((
			GameStatePlugin,
			GameSchedulePlugin,
			AssetsPlugin,
			GameCameraPlugin,
			PlayerPlugin,
			BallPlugin,
			PitchPlugin,
			GameControlPlugin,
			GameGizmosPlugin,
			KitPlugin,
			FlagPlugin,
			//AnimatedMeshPlugin,
		))
		.insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
    .insert_resource(GlobalAmbientLight {
        color: WHITE.into(),
        brightness: 1_000.0,
        ..default()
    })
		.insert_resource(DirectionalLightShadowMap { size: 4096 })
		.add_systems(Startup, init_lights)
		.run();
}



fn init_lights(mut commands:Commands){

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
