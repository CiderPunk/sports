use std::collections::VecDeque;

use bevy::{ecs::{lifecycle::HookContext, world::DeferredWorld}, prelude::*};

use crate::game_schedule::GameSchedule;

pub struct ThinkDistributorPlugin;

impl Plugin for ThinkDistributorPlugin{
	fn build(&self, app: &mut App) {
		app
			.init_resource::<ThinkScheduler>()
			.add_systems(FixedUpdate, distribute_thinks.in_set(GameSchedule::PreMovement))
			;
	}
}

const MAX_THINKS_PER_FRAME:usize = 4;

fn distribute_thinks(
	mut commands:Commands,
	mut schedule:ResMut<ThinkScheduler>,
	previous_thinkers:Query<Entity, With<ThinkNext>>,
){
	//remove last frames thinks
	for entity in previous_thinkers{
		commands.entity(entity).remove::<ThinkNext>();
	}

	let thinkers_to_think = MAX_THINKS_PER_FRAME.min(schedule.queue.len());
  for _ in 0 .. thinkers_to_think{
		if let Some(entity) = schedule.queue.pop_front(){
			commands.entity(entity).insert(ThinkNext);
			schedule.queue.push_back(entity);
		}
	}  
}

#[derive(Resource, Default)]
pub struct ThinkScheduler{
	pub queue:VecDeque<Entity>,
}



#[derive(Component)]
#[component(on_add = on_thinker_added)]
#[component(on_remove = on_thinker_removed)]
pub struct Thinker;

fn on_thinker_added(mut world: DeferredWorld, context:HookContext)  {
	let entity= context.entity;
	if let Some(mut scheduler) =  world.get_resource_mut::<ThinkScheduler>(){
		info!("Added thinker {}", entity);
		scheduler.queue.push_back(entity);
	}
}
fn on_thinker_removed(mut world: DeferredWorld, context:HookContext)  {
	let entity= context.entity;
	if let Some(mut scheduler) =  world.get_resource_mut::<ThinkScheduler>(){
		info!("removed thinker {}", entity);
		scheduler.queue.retain(|&e| e != entity);
	}
}



#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ThinkNext;


