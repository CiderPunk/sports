use std::collections::HashMap;

use bevy::{color::palettes::css::{BLUE, GREEN, PINK, RED, WHITE, YELLOW}, prelude::*};
use strum::VariantArray;
use strum_macros::VariantArray;

pub struct GameGizmosPlugin;
impl Plugin for GameGizmosPlugin{
	fn build(&self, app: &mut App) {
		app
			.add_message::<GizmoSpawnMessage>()
			.add_systems(Startup, init_gizmos)
			.add_systems(Update, (spawn_gizmos, despawn_gizmos).chain())
			;
	}
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, VariantArray)]
pub enum GizmoColour{
	Red,
	Blue, 
	White,
	Green, 
	Yellow,
	Pink, 
}

impl GizmoColour{
	pub const fn to_srgba(&self)-> Srgba{
		match self{
			GizmoColour::Red => RED,
			GizmoColour::Blue => BLUE,
			GizmoColour::White => WHITE,
			GizmoColour::Green => GREEN,
			GizmoColour::Yellow => YELLOW,
			GizmoColour::Pink => PINK,
		}

	}

}


#[derive(Message)]
pub struct GizmoSpawnMessage{
	transform:Transform,
	colour:GizmoColour,
}

impl GizmoSpawnMessage{
	pub fn new(transform:Transform, colour:GizmoColour)->Self{
		Self{ transform, colour }
	}
}

#[derive(Component)]
struct TimeToLive(Timer);

#[derive(Resource)]
pub struct GameGizmoStore{
	pub cross_colours:HashMap<GizmoColour, Handle<GizmoAsset>>,
	pub sphere_colours:HashMap<GizmoColour, Handle<GizmoAsset>>,
}


fn init_gizmos(
	mut commands:Commands,
	mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
	mut spawn_writer: MessageWriter<GizmoSpawnMessage>
){
	let mut cross_colours:HashMap<GizmoColour, Handle<GizmoAsset>> = HashMap::new();
	let mut sphere_colours:HashMap<GizmoColour, Handle<GizmoAsset>> = HashMap::new();
	for colour in GizmoColour::VARIANTS{
		let mut cross = GizmoAsset::new();
		cross.cross(Isometry3d::IDENTITY, 1.0, colour.to_srgba());
		cross_colours.insert(colour.clone(), gizmo_assets.add(cross));

		let mut sphere = GizmoAsset::new();
		sphere.sphere(Isometry3d::IDENTITY, 1.0, colour.to_srgba());
		sphere_colours.insert(colour.clone(), gizmo_assets.add(sphere));

	}

	commands.insert_resource(
		GameGizmoStore{ 
			cross_colours, 
			sphere_colours,
		 });
	//spawn_writer.write(GizmoSpawnMessage::new(Transform::from_xyz(0.,0.,0.), GizmoColour::Blue));
}

fn spawn_gizmos(
	mut spawn_reader:MessageReader<GizmoSpawnMessage>,
	mut commands:Commands,
	game_gizmos:Res<GameGizmoStore>,
){
	for spawn_message in spawn_reader.read(){
		let colour_handle = game_gizmos.cross_colours.get(&spawn_message.colour).expect("Missing colour gizmo");
		commands.spawn((
			Gizmo{
				handle:colour_handle.clone(),
				..default()
			},
			TimeToLive(Timer::from_seconds(5., TimerMode::Once)),
			spawn_message.transform,
		));
	}
}


fn despawn_gizmos(
	query:Query<(&mut TimeToLive, Entity)>,
	mut commands:Commands,
	time:Res<Time>,
){
	for (mut ttl, entity) in query{
		ttl.0.tick(time.delta());
		if ttl.0.is_finished(){
			commands.entity(entity).despawn();
		}
	}
}
