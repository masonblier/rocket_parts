use crate::loading::TextureAssets;
use crate::game_state::GameState;

use bevy::{
    prelude::*,
    pbr::{ExtendedMaterial, MaterialExtension, OpaqueRendererMethod},
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Default)]
pub struct TerrainMaterialPlugin;

// system state
#[derive(Default, Resource)]
pub struct TerrainMaterialState {
    pub terrain_material: Handle<ExtendedMaterial<StandardMaterial, TerrainMaterial>>,
}

// material layers stacked in 256x1024 texture
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct TerrainMaterial {
    #[uniform(100)]
    quantize_steps: u32,
    #[texture(101, dimension = "2d_array")]
    #[sampler(102)]
    array_texture: Handle<Image>,
}

impl Plugin for TerrainMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainMaterialState::default());
        app.add_plugins((MaterialPlugin::<ExtendedMaterial<StandardMaterial, TerrainMaterial>>::default(),));
        app.add_systems(OnEnter(GameState::WorldLoading), setup_terrain_material);
    }
}

// material shaders
impl MaterialExtension for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }
    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }
}

fn setup_terrain_material(
    mut terrain_materials: ResMut<TerrainMaterialState>,
    mut images: ResMut<Assets<Image>>,
    texture_assets: Res<TextureAssets>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
) {
    terrain_materials.terrain_material = 
        materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: Color::RED,
                // can be used in forward or deferred mode.
                opaque_render_method: OpaqueRendererMethod::Auto,
                // in deferred mode, only the PbrInput can be modified (uvs, color and other material properties),
                // in forward mode, the output can also be modified after lighting is applied.
                // see the fragment shader `extended_material.wgsl` for more info.
                // Note: to run in deferred mode, you must also add a `DeferredPrepass` component to the camera and either
                // change the above to `OpaqueRendererMethod::Deferred` or add the `DefaultOpaqueRendererMethod` resource.
                ..Default::default()
            },
            extension: TerrainMaterial {
                quantize_steps: 3,
                array_texture: texture_assets.texture_array.clone(),
            },
        });

    // configure texture as stacked 2d array
    let image = images.get_mut(&texture_assets.texture_array).unwrap();
    let array_layers = 4;
    image.reinterpret_stacked_2d_as_array(array_layers);
}
