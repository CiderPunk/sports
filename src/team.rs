use core::slice;
use bevy::{color::palettes::{css::{BLACK, BLUE, DARK_BLUE, LIMEGREEN, RED, WHITE, YELLOW}, tailwind::GRAY_700}, prelude::*};
use crate::kit::{KitColour, KitConfiguration};

pub struct TeamPlugin;

impl Plugin for TeamPlugin{
	fn build(&self, app: &mut App) {
		app
			.add_systems(Startup, init_teams)
			;
	}
}


#[derive(Debug,Component, Clone)]
pub struct Team{
	pub top:bool,
	pub name:String, 
	pub kit:KitConfiguration,
	pub goalie_kit:KitConfiguration,
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
			top: true,
			name: String::from("Reds"),
			kit:KitConfiguration { 
				pattern: crate::kit::KitPattern::Quatered, 
				colour_primary: KitColour::from_srgba(RED),
				colour_secondary: KitColour::from_srgba(GRAY_700),
				colour_tertiary: KitColour::from_srgba(WHITE), 
				shirt_number: 1 
			},
			goalie_kit:KitConfiguration { 
				pattern: crate::kit::KitPattern::Solid, 
				colour_primary: KitColour::from_srgba(YELLOW),
				colour_secondary: KitColour::from_srgba(YELLOW),
				colour_tertiary: KitColour::from_srgba(RED),  
				shirt_number: 1
			},
		}
	));
	commands.spawn(
		Team{
			top:false,
			name: String::from("Blues"),
			kit:KitConfiguration { 
				pattern: crate::kit::KitPattern::Solid, 
				colour_primary: KitColour::from_srgba(BLUE),
				colour_secondary: KitColour::from_srgba(DARK_BLUE),
				colour_tertiary: KitColour::from_srgba(WHITE), 
				shirt_number: 1 
			},
			goalie_kit:KitConfiguration { 
				pattern: crate::kit::KitPattern::Solid, 
				colour_primary: KitColour::from_srgba(LIMEGREEN),
				colour_secondary: KitColour::from_srgba(LIMEGREEN),
				colour_tertiary: KitColour::from_srgba(BLUE),  
				shirt_number: 1
			},
		}
	);
}