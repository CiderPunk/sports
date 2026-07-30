use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::{marker::PhantomData, time::Duration};

pub struct AnimationManagerPlugin;

impl Plugin for AnimationManagerPlugin {
	fn build(&self, app: &mut App) {
		//app.add_systems(Update, restart_visible_animations)
		;
		


	}
}

#[derive(Component, Debug)]
pub struct Animator {
	pub entity: Entity,
}

impl Animator{
}

fn restart_visible_animations(
 	mut query: Query<(&mut AnimationTransitions, &mut AnimationPlayer, &ViewVisibility), Changed<ViewVisibility>>,
){
	for (transitions, mut player, view_visibility) in query.iter_mut(){

		if view_visibility.get() 
			&& let Some(anim) =  transitions.get_main_animation() 
			&& let Some(active_animation) = player.animation_mut(anim){
				
				info!("Restarting animation");
				active_animation.resume();
		}
	}
}

#[derive(Resource)]
pub struct MeshAnimations<T: Component> {
	pub animations: Vec<AnimationNodeIndex>,
	pub graph_handle: Handle<AnimationGraph>,
	pub _marker: PhantomData<T>,
}


#[derive(SystemParam)]
pub struct AnimationManager<'w, 's, T:Component + 'static> {
	pub commands: Commands<'w, 's>,
	pub gltfs: Res<'w, Assets<Gltf>>,
	pub graphs: ResMut<'w, Assets<AnimationGraph>>,
	pub anim_resource: Option<Res<'w,  MeshAnimations<T>>>,
	pub children_query: Query<'w, 's, &'static Children>,
	pub animator_query:Query<'w, 's, &'static Animator>,
	pub anim_player_queries: ParamSet<'w, 's, (
	 	Query<'w, 's, &'static mut AnimationPlayer>,
		Query<'w, 's, (&'static mut AnimationPlayer, &'static mut AnimationTransitions)>,
	)>,
	pub _marker: PhantomData<fn() -> T>,
}

impl<'w, 's, T:Component +'static> AnimationManager<'w, 's, T> {
	pub fn create_graph(
		&mut self,
		gltf_handle: Handle<Gltf>,
		clip_names: &[&str],
	) {
		let gltf = self.gltfs.get(&gltf_handle).expect("Missing gltf");

		let clips = clip_names.iter().map(|name| {
			gltf.named_animations
				.get(*name)
				.cloned()
				.unwrap_or_else(|| panic!("Animation clip {} missing in GLTF", name))
		});

		let (graph, node_indices) = AnimationGraph::from_clips(clips);
		let graph_handle = self.graphs.add(graph);

		self.commands.insert_resource(MeshAnimations::<T> {
			animations: node_indices,
			graph_handle,
			_marker: PhantomData,
		});
	}

	pub fn attach_animation(
			&mut self,
			root_entity: Entity,
			start_index: usize,
	){
		let animations = self.anim_resource.as_ref().unwrap_or_else(||{ panic!("missing animation resources") });
		for descendant in self.children_query.iter_descendants(root_entity) {
			if let Ok(mut anim_player) = self.anim_player_queries.p0().get_mut(descendant) {
				let mut transitions = AnimationTransitions::new();
				let anim = animations.animations.get(start_index).expect("Out of bounds start animation index");
				transitions
					.play(
						&mut anim_player,
						anim.clone(),
						Duration::ZERO,
					)
					.repeat();

				self.commands.entity(descendant).insert((
					AnimationGraphHandle(animations.graph_handle.clone()),
					transitions,
				));

				self.commands
					.entity(root_entity)
					.insert(Animator { entity: descendant });
				break;
			}
		}
	}

	pub fn set_animation(&mut self, root_entity:Entity, index:usize, transition_time:f32, speed:f32, repeat:bool  ){
		let animations = self.anim_resource.as_ref().unwrap_or_else(||{ panic!("missing animation resources") });
		let anim = animations.animations.get(index).unwrap_or_else(||{ panic!("Out of bounds animation index") });
		if let Ok(animator)  = self.animator_query.get(root_entity) {
			if let Ok((mut player, mut transition)) = self.anim_player_queries.p1().get_mut(animator.entity){
				if transition.get_main_animation() != Some(*anim){
					transition.play(&mut player, *anim, Duration::from_secs_f32(transition_time)).set_speed(speed).repeat();
				}
				else if let Some(active_animation) = player.animation_mut(*anim){
					active_animation.set_speed(speed);
				}
			};
		};
	}

}
