use mg_core::*;
use mg_render::{
    geometry::Geometry, graphics::Graphics, material, material::Material, mesh::Mesh,
    texture::Texture,
};

pub struct Assets {
    pub std_wheel: Mesh,
    pub gyro_kart: Mesh,
}

impl Assets {
    pub fn new(graphics: &Graphics) -> Assets {
        let palette = Arc::new(Material::new(
            graphics,
            material::Params {
                name: "palette".to_owned(),
                albedo_texture: Texture::create_image_texture(
                    &graphics,
                    "paletteA",
                    include_bytes!("textures/PKG_.png"),
                ),
                albedo_sampler: None,
                emission_texture: Texture::create_image_texture(
                    &graphics,
                    "paletteE",
                    include_bytes!("textures/paletteE.png"),
                ),
                emission_sampler: None,
            },
        ));

        Geometry::from_gltf("glbs/PKG_D.1_10kCandles/NewSponza_4_Combined_gltf");

        let gyro_kart = Mesh {
            name: "gyro kart".to_owned(),
            geometry: gyro_kart_geo,
            material: palette.clone(),
        };
        let std_wheel = Mesh {
            name: "std wheel".to_owned(),
            geometry: std_wheel_geo,
            material: palette.clone(),
        };

        Self {
            gyro_kart,
            std_wheel,
        }
    }
}
