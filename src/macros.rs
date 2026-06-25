#[macro_export]
macro_rules! get_gltf_primative{
  ($gltf_meshes:ident, $models:ident, $name:literal)=>{
   &$gltf_meshes.get( 
      $models.named_meshes.get($name)
      .ok_or(concat!("couldn't get ", $name, " mesh"))?,)
      .ok_or(concat!("couldn't get ", $name, " mesh data"))?
    .primitives[0]
  }
}
