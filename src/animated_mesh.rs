
use std::{marker::PhantomData, time::Duration};

use bevy::{ecs::system::{StaticSystemParam, SystemParam}, math::usize, prelude::*};
pub struct AnimatedMeshPlugin;

impl Plugin for AnimatedMeshPlugin{
	fn build(&self, app: &mut App) {

	}
}


#[derive(Component, Debug)]
pub struct Animator{
	entity:Entity,
}

#[derive(Resource)]
pub struct MeshAnimations<T: Component> {
	animations: Vec<AnimationNodeIndex>,
	graph_handle: Handle<AnimationGraph>,
	_marker: PhantomData<T>,
}  

#[derive(SystemParam)]
pub struct AnimationManager<'w, 's, T: 'static> {
    pub commands: Commands<'w, 's>,
    pub gltfs: Res<'w, Assets<Gltf>>,
    pub graphs: ResMut<'w, Assets<AnimationGraph>>,
    pub anim_resource: Option<Res<'w, MeshAnimations<T>>>,
    pub children_query: Query<'w, 's, &'s Children>,
    pub anim_player_query: Query<'w, 's, &'w mut AnimationPlayer>,
    pub _marker: PhantomData<fn() -> T>,
}

impl<'w,'s,T:Component +'static> AnimationManager<'w,'s,T>{
	pub fn create_graph(&mut self, gltf_handle:Handle<Gltf>, clip_names: &[&str]){
		let gltf = self.gltfs.get(&gltf_handle).expect("Missing gltf");
		let clips = clip_names.iter().map(|name|{
			gltf.named_animations.get(*name).cloned()
				.unwrap_or_else(|| panic!("Animation clip {} missing in GLTF", name))
		});
		let (graph, node_indices) = AnimationGraph::from_clips(clips);
		let graph_handle = self.graphs.add(graph);
     // Insert the unique resource specifically for component type T
		self.commands.insert_resource(MeshAnimations::<T> {
			animations: node_indices,
			graph_handle,
			_marker: PhantomData,
		});
	}

	pub fn attach_animation(&mut self,  root_entity:Entity, start_index:usize){
		let Some(ref mesh_anims) = self.anim_resource.as_ref().map(|s| s.as_ref()) else {
			warn!("MeshAnimations resource for this type is not yet loaded.");
			return;
		};

		for descendant in self.children_query.iter_descendants(root_entity) {
			if let Ok(mut anim_player) = self.anim_player_query.get_mut(descendant) {
				let mut transitions = AnimationTransitions::new();
				// Play and repeat the primary/first default animation node
				transitions
					.play(&mut anim_player, mesh_anims.animations[start_index], Duration::ZERO)
					.repeat();

				self.commands
					.entity(descendant)
					.insert((AnimationGraphHandle(mesh_anims.graph_handle.clone()), transitions));

				self.commands
					.entity(root_entity).insert(Animator{ entity: descendant });
				
				break;
			}
		}
	}
} 

