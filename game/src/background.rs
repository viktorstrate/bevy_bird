use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_asset::{PrepareAssetError, RenderAsset, RenderAssets},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingResource, BindingType, SamplerBindingType, ShaderStages,
            TextureSampleType, TextureViewDimension,
        },
        renderer::RenderDevice,
    },
    sprite::{Material2d, Material2dPipeline, Material2dPlugin, MaterialMesh2dBundle},
};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<BackgroundMaterial>::default());
        app.add_startup_system(setup_background);
    }
}

fn setup_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let background_material = BackgroundMaterial {
        texture: asset_server.load("textures/paint-seamless.png"),
    };

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        material: materials.add(background_material),
        transform: Transform::from_xyz(0., 0., -1.).with_scale(Vec3::splat(2.)),
        ..Default::default()
    });
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "a1f72471-e24f-485d-bdfc-421f4dd32e20"]
pub struct BackgroundMaterial {
    // Texture to blend into the background
    pub texture: Handle<Image>,
}

pub struct GpuBackgroundMaterial {
    bind_group: BindGroup,
}

impl RenderAsset for BackgroundMaterial {
    type ExtractedAsset = BackgroundMaterial;

    type PreparedAsset = GpuBackgroundMaterial;

    type Param = (
        SRes<RenderDevice>,
        SRes<Material2dPipeline<BackgroundMaterial>>,
        SRes<RenderAssets<Image>>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        material: Self::ExtractedAsset,
        (render_device, background_pipeline, gpu_images): &mut bevy::ecs::system::SystemParamItem<
            Self::Param,
        >,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        let (texture_view, sampler) = if let Some(gpu_image) = gpu_images.get(&material.texture) {
            (&gpu_image.texture_view, &gpu_image.sampler)
        } else {
            return Err(PrepareAssetError::RetryNextUpdate(material));
        };

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
            label: Some("background_material_bind_group"),
            layout: &background_pipeline.material2d_layout,
        });

        Ok(GpuBackgroundMaterial { bind_group })
    }
}

impl Material2d for BackgroundMaterial {
    fn bind_group(
        render_asset: &<Self as bevy::render::render_asset::RenderAsset>::PreparedAsset,
    ) -> &bevy::render::render_resource::BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(
        render_device: &bevy::render::renderer::RenderDevice,
    ) -> bevy::render::render_resource::BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("background_material_layout"),
            entries: &[
                // Texture
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                // Texture Sampler
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    fn vertex_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/background.wgsl"))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/background.wgsl"))
    }
}
