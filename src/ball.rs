use bevy::prelude::*;

use bevy_asset_loader::prelude::*;
use crate::{assets::AssetLoadState, game_state::GameState};


const BALL_SCALE: f32 = 0.5;

pub struct BallPlugin;
impl Plugin for BallPlugin{
	fn build(&self, app: &mut App) {
		app
			.configure_loading_state(
				LoadingStateConfig::new(AssetLoadState::Startup)
				.load_collection::<BallAssets>(),
			)
			.add_systems(OnEnter(GameState::Playing), spawn_ball);
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
		WorldAssetRoot(ball_assets.ball_scene.clone()),
		Transform::from_translation(Vec3::new(0., 0.5* BALL_SCALE,0.)).with_scale(Vec3::splat(BALL_SCALE)),
	));

}
