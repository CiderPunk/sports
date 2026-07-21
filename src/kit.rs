use bevy::{platform::collections::HashMap, prelude::*, reflect::TypeData};
use bevy_asset_loader::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use rand::seq::IndexedRandom;
use strum_macros::VariantArray;

use crate::assets::AssetLoadState;
pub struct KitPlugin;

impl Plugin for KitPlugin{
	fn build(&self, app: &mut App) {
		app.configure_loading_state(
				LoadingStateConfig::new(AssetLoadState::Startup)
				.load_collection::<KitAssets>(),
			)
			.init_resource::<KitFactory>()
			;
	}
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct KitColour(pub [u8;4]);

impl KitColour{
	pub fn from_srgba(colour:Srgba) -> Self{
		Self(colour.to_u8_array())
	}
	pub fn to_srgba(self) -> Srgba{
		Srgba::from_u8_array(self.0)
	}
}


#[derive(AssetCollection, Resource, Default)]
pub struct KitAssets {
  #[asset(path = "textures/kit.png")]
  pub default_kit: Handle<Image>,
  #[asset(path = "textures/kit-quarter.png")]
  pub kit_quatered: Handle<Image>,
  #[asset(path = "textures/kit-stripe.png")]
  pub kit_striped: Handle<Image>,
  
	#[asset(path = "textures/skin.png")]
  pub default_skin: Handle<Image>,
	#[asset(paths("textures/skin-matt.png","textures/skin-jam.png", "textures/skin-tezz.png"), collection(typed))]
	skins: Vec<Handle<Image>>,
}


#[derive(Clone, Copy, Debug, VariantArray, Hash, PartialEq, Eq)]
pub enum KitPattern{
	Solid,
	Striped,
	Quatered, 
}




#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct KitConfiguration{
	pub pattern:KitPattern,
	pub colour_primary: KitColour,
	pub colour_secondary: KitColour,
	pub colour_tertiary: KitColour,
	pub shirt_number:u8,
}

#[derive(Resource, Default)]
pub struct KitFactory{
	cache: HashMap<KitConfiguration, Handle<Image>>,
}

impl KitFactory{
	pub fn get_or_generate(
		&mut self, 
		config: KitConfiguration,
		kit_assets:&Res<KitAssets>,
		mut images: ResMut<Assets<Image>>, 
		mut rng: Single<&mut WyRand, With<GlobalRng>>,
	) ->Handle<Image>{

		if let Some(texture) = self.cache.get(&config){
			return texture.clone();
		};
		info!("Generating kit");

		let kit_image_id = match config.pattern{
				KitPattern::Solid => kit_assets.default_kit.id(),
				KitPattern::Striped => kit_assets.kit_striped.id(),
				KitPattern::Quatered => kit_assets.kit_quatered.id(),
		};

		let mut skin_texture = images.get(kit_assets.skins.choose(&mut rng).expect("failed picking a random skin").id()).expect("Missing player skin texture").clone();
		let kit_image = images.get(kit_image_id).expect("Missing kit texture").clone();
	
	 	let mut skin_data = skin_texture.data.take().expect("Skin texture lacks raw data bytes");   
		let kit_data = kit_image.data.as_ref().expect("Kit texture lacks raw data");

		let skin_chunks = skin_data.chunks_exact_mut(4);
		let kit_chunks = kit_data.chunks_exact(4);

		for (skin_pixel, kit_pixel) in skin_chunks.zip(kit_chunks){
			//skin_pixel.copy_from_slice(&config.colour_primary.0);
			if kit_pixel[2] > 200{
				skin_pixel.copy_from_slice(&config.colour_primary.0);
			}
			if kit_pixel[0] > 200{
				skin_pixel.copy_from_slice(&config.colour_secondary.0);
			}
			if kit_pixel[1] > 200{
				skin_pixel.copy_from_slice(&config.colour_tertiary.0);
			}
		}
		skin_texture.data = Some(skin_data);
		let handle = images.add(skin_texture);
		self.cache.insert(config, handle.clone());

		info!("Kit complete");
		handle
		//kit_assets.default_skin.clone()
	}
}