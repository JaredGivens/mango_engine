use mg_core::*;
use mg_render::{
    geometry::Geometry, gltf_loader, graphics::Graphics, material, material::Material, mesh::Mesh,
    texture::Texture,
};

pub struct Assets {
    defaults: material::Defaults,
    pub meshes: Vec<Mesh>,
}

impl Assets {
    pub fn new(graphics: &Graphics) -> Assets {
        let defaults = material::Defaults::new(graphics);
        let meshes = gltf_loader::meshes_from_separated(
            graphics,
            &defaults,
            "../../assets/PKG_A_Curtains/",
            "NewSponza_Curtains_glTF",
        );

        Self { defaults, meshes }
    }
}
