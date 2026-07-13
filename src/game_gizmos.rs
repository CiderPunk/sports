use std::collections::HashMap;

use bevy::{color::palettes::css::{BLUE, CRIMSON, GREEN, PINK, RED, WHITE, YELLOW}, prelude::*};
use strum::VariantArray;
use strum_macros::VariantArray;

pub struct GameGizmosPlugin;
impl Plugin for GameGizmosPlugin{
	fn build(&self, app: &mut App) {
		app
			.add_message::<GizmoSpawnMessage>()
			.add_systems(Startup, init_gizmos)
			//.add_systems(Update, (spawn_gizmos, despawn_gizmos).chain())
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

#[derive(Resource)]
struct GameGizmos{
	cross_colours:HashMap<GizmoColour, Handle<GizmoAsset>>,
	ttl:Timer,
}


fn init_gizmos(
	mut commands:Commands,
	mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
	mut spawn_writer: MessageWriter<GizmoSpawnMessage>
){
	let mut cross_colours:HashMap<GizmoColour, Handle<GizmoAsset>> = HashMap::new();
	for colour in GizmoColour::VARIANTS{
		let mut cross = GizmoAsset::new();
		cross.cross(Isometry3d::IDENTITY, 1.0, colour.to_srgba());
		cross_colours.insert(colour.clone(), gizmo_assets.add(cross));
	}

	commands.insert_resource(GameGizmos{ cross_colours, ttl:Timer::from_seconds(5., TimerMode::Once) });
	spawn_writer.write(GizmoSpawnMessage::new(Transform::from_xyz(0.,0.,0.), GizmoColour::Blue));
}

fn spawn_gizmos(
	mut spawn_reader:MessageReader<GizmoSpawnMessage>,
	mut commands:Commands,
	game_gizmos:Res<GameGizmos>,
){
	for spawn_message in spawn_reader.read(){
		let colour_handle = game_gizmos.cross_colours.get(&spawn_message.colour).expect("Missing colour gizmo");
		commands.spawn((
			Gizmo{
				handle:colour_handle.clone(),
				..default()
			},
			spawn_message.transform,
		));
	}
}


fn despawn_gizmos(
	query:Query<(&mut GameGizmos, Entity)>,
	mut commands:Commands,
	time:Res<Time>,
){
	for (mut gizmo, entity) in query{
		gizmo.ttl.tick(time.delta());
		if gizmo.ttl.is_finished(){
			commands.entity(entity).despawn();
		}
	}
}
