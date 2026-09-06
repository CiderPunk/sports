use core::slice;
use bevy::{color::palettes::css::{BLACK, BLUE, RED, WHITE}, prelude::*};
use crate::kit::{KitColour, KitConfiguration};

pub struct TeamPlugin;

impl Plugin for TeamPlugin{
	fn build(&self, app: &mut App) {
		app
			.add_systems(Startup, init_teams)
			;
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TeamSide{
	North,
	South,
}

#[derive(Debug,Component, Clone)]
pub struct Team{
	pub side:TeamSide,
	pub name:String, 
	pub kit:KitConfiguration,
}

#[derive(Component)]
#[relationship(relationship_target = TeamMembers)]
pub struct TeamMember(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = TeamMember)]
pub struct TeamMembers(Vec<Entity>);

#[derive(Component)]
pub struct PlayerControlled;

impl<'a> IntoIterator for &'a TeamMembers {
    type Item = <Self::IntoIter as Iterator>::Item;

    type IntoIter = slice::Iter<'a, Entity>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

fn init_teams(
	mut commands:Commands
){
	commands.spawn((
		PlayerControlled,
		Team{
			side: TeamSide::North,
			name: String::from("Reds"),
			kit:KitConfiguration { 
				pattern: crate::kit::KitPattern::Quatered, 
				colour_primary: KitColour::from_srgba(RED),
				colour_secondary: KitColour::from_srgba(BLACK),
				colour_tertiary: KitColour::from_srgba(WHITE), 
				shirt_number: 1 
			},
		}
	));
	commands.spawn(
		Team{
			side: TeamSide::South,
			name: String::from("Blues"),
			kit:KitConfiguration { 
				pattern: crate::kit::KitPattern::Solid, 
				colour_primary: KitColour::from_srgba(BLUE),
				colour_secondary: KitColour::from_srgba(BLACK),
				colour_tertiary: KitColour::from_srgba(WHITE), 
				shirt_number: 1 
			},
		}
	);
}