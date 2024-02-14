use mg_core::*;
use mg_render::{
    geometry::Geometry, gltf_loader, wgpu_ctx::WgpuContext, material, material::Material,
    texture::Texture, mesh::Mesh, mesh
};

pub struct Assets {
    defaults: material::Defaults,
    pub meshes: Vec<Mesh>,
}

impl Assets {
    pub fn new(w_ctx: &WgpuContext) -> Assets {
        // TODO: to make the program not crash give a path/file that doesnt exist
        let defaults = material::Defaults::new(w_ctx);
        let meshes = gltf_loader::meshes_from_separated(
            w_ctx,
            &defaults,
            "assets/PKG_A_Curtains/",
            "NewSponza_Curtains_glTF",
        );

        Self { defaults, meshes }
    }
}
