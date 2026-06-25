mod player;
mod assets;
mod macros;
use bevy::{color::palettes::css::WHITE, core_pipeline::tonemapping::Tonemapping, prelude::*};
use crate::{assets::AssetsPlugin, player::PlayerPlugin};

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
			AssetsPlugin,
			PlayerPlugin,

		))
		.insert_resource(ClearColor(Color::srgb(0., 0., 0.)))
    .insert_resource(GlobalAmbientLight {
        color: WHITE.into(),
        brightness: 40.0,
        ..default()
    })
		.add_systems(Startup, init_camera)
		.run();
}



fn init_camera(mut commands:Commands){
  commands.spawn((

    Camera3d{
      
      ..default()
    },
    Camera {
      order: 1, 
      ..default()
    },
    Tonemapping::BlenderFilmic,
    Transform::from_translation(Vec3::new(0.,0.,100.)).looking_at(Vec3::ZERO, Vec3::Y),
  ));
}
