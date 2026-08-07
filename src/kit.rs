use bevy::{ecs::system::SystemParam, platform::collections::HashMap, prelude::*, reflect::TypeData};
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
			.init_resource::<KitCache>()
			.init_resource::<FlagCache>()
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
  pub kit_default: Handle<Image>,
  #[asset(path = "textures/kit-quarter.png")]
  pub kit_quatered: Handle<Image>,
  #[asset(path = "textures/kit-stripe.png")]
  pub kit_striped: Handle<Image>,
  
  #[asset(path = "textures/flag.png")]
  pub flag_default: Handle<Image>,
  #[asset(path = "textures/flag-quarter.png")]
  pub flag_quatered: Handle<Image>,
  #[asset(path = "textures/flag-stripe.png")]
  pub flag_striped: Handle<Image>,
  


	#[asset(path = "textures/skin.png")]
  pub default_skin: Handle<Image>,
	#[asset(paths("textures/skin-matt.png","textures/skin-jam.png", "textures/skin-tezz.png"), collection(typed))]
	kit_skins: Vec<Handle<Image>>,
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
pub struct KitCache(HashMap<KitConfiguration, Handle<Image>>);

#[derive(Resource, Default)]
pub struct FlagCache(HashMap<KitConfiguration, Handle<Image>>);


#[derive(SystemParam)]
pub struct KitGenerator<'w, 's> {
	flag_cache: ResMut<'w, FlagCache>,
	kit_cache: ResMut<'w, KitCache>,
	kit_assets: Res<'w, KitAssets>,
	images: ResMut<'w, Assets<Image>>,
	rng: Single<'w, 's, &'static mut WyRand, With<GlobalRng>>,
	materials: ResMut<'w, Assets<StandardMaterial>>,
}

impl<'w,'s> KitGenerator<'w,'s>{

	pub fn make_material(
		&mut self, 
		base_material:Handle<StandardMaterial>,
		image:Handle<Image>,
	)-> Handle<StandardMaterial>{
		if let Some(base_material) = self.materials.get(base_material.id()){
			let mut material = base_material.clone();
				material.base_color_texture = Some(image.clone());
				self.materials.add(material)
		} else {
			self.materials.add(StandardMaterial {
				base_color_texture: Some(image.clone()),
				..default()
			})
		}
	}



	pub fn get_or_generate_flag(
		&mut self, 
		config: KitConfiguration,
	)->Handle<Image>{
		if let Some(texture) = self.flag_cache.0.get(&config){
			return texture.clone();
		};
		info!("Generating flag");
		let kit_image_id = match config.pattern{
			KitPattern::Solid => self.kit_assets.flag_default.id(),
			KitPattern::Striped => self.kit_assets.flag_striped.id(),
			KitPattern::Quatered => self.kit_assets.flag_quatered.id(),
		};

		let mut flag_texture = self.images.get(kit_image_id).expect("Missing player skin texture").clone();
		let mut data = flag_texture.data.take().expect("Flag texture lacks raw data bytes");
		let chunks = data.chunks_exact_mut(4);

		for pixel in chunks{
			//skin_pixel.copy_from_slice(&config.colour_primary.0);
			if pixel[2] > 200{
				pixel.copy_from_slice(&config.colour_primary.0);
			}
			else if pixel[0] > 200{
				pixel.copy_from_slice(&config.colour_secondary.0);
			}
			else if pixel[1] > 200{
				pixel.copy_from_slice(&config.colour_tertiary.0);
			}
		};
		flag_texture.data = Some(data);
		let handle = self.images.add(flag_texture);
		self.flag_cache.0.insert(config, handle.clone());
		info!("Kit complete");
		handle
	}




	pub fn get_or_generate_kit(
		&mut self, 
		config: KitConfiguration,
	) ->Handle<Image>{

		if let Some(texture) = self.kit_cache.0.get(&config){
			return texture.clone();
		};
		info!("Generating kit");

		let rng_mut = &mut **self.rng;
		
		let kit_image_id = match config.pattern{
			KitPattern::Solid => self.kit_assets.kit_default.id(),
			KitPattern::Striped => self.kit_assets.kit_striped.id(),
			KitPattern::Quatered => self.kit_assets.kit_quatered.id(),
		};

		let mut skin_texture = self.images.get(self.kit_assets.kit_skins.choose(rng_mut).expect("failed picking a random skin").id()).expect("Missing player skin texture").clone();
		let kit_image = self.images.get(kit_image_id).expect("Missing kit texture").clone();
	
	 	let mut skin_data = skin_texture.data.take().expect("Skin texture lacks raw data bytes");   
		let kit_data = kit_image.data.as_ref().expect("Kit texture lacks raw data");

		let skin_chunks = skin_data.chunks_exact_mut(4);
		let kit_chunks = kit_data.chunks_exact(4);

		for (skin_pixel, kit_pixel) in skin_chunks.zip(kit_chunks){
			//skin_pixel.copy_from_slice(&config.colour_primary.0);
			if kit_pixel[2] > 200{
				skin_pixel.copy_from_slice(&config.colour_primary.0);
			}
		 	else	if kit_pixel[0] > 200{
				skin_pixel.copy_from_slice(&config.colour_secondary.0);
			}
			else if kit_pixel[1] > 200{
				skin_pixel.copy_from_slice(&config.colour_tertiary.0);
			}
		}
		skin_texture.data = Some(skin_data);
		let handle = self.images.add(skin_texture);
		self.kit_cache.0.insert(config, handle.clone());

		info!("Kit complete");
		handle
		//kit_assets.default_skin.clone()
	}
}