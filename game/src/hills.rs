use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        render_asset::{PrepareAssetError, RenderAsset, RenderAssets},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
            BindGroupLayoutEntry, BindingResource, BindingType, PrimitiveTopology,
            RenderPipelineDescriptor, SamplerBindingType, ShaderStages, TextureSampleType,
            TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat,
            VertexStepMode,
        },
        renderer::RenderDevice,
    },
    sprite::{Material2dPipeline, Material2dPlugin, SpecializedMaterial2d},
};

pub fn hills_mesh() -> Mesh {
    // Generate vertex positions
    const STEPS: i32 = 75;
    let mut v_pos = vec![];

    for i in 0..=STEPS {
        let x_offset = (i as f32) / (STEPS as f32) - 0.5;
        v_pos.push([x_offset, 0.]);
        v_pos.push([
            x_offset,
            (i as f32 / (STEPS as f32) * std::f32::consts::TAU).sin() * 0.1 + 1.,
        ]);
    }

    // Generate indices for vertex positions
    let mut indices = vec![];
    for i in 0..=(STEPS - 1) {
        let x = (i * 2) as u32;
        indices.extend_from_slice(&[x, x + 3, x + 1]);
        indices.extend_from_slice(&[x, x + 2, x + 3]);
    }

    // Save vertex data to a new mesh
    let mut hills_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    hills_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    hills_mesh.set_indices(Some(Indices::U32(indices)));

    hills_mesh
}

// Plugin to register the custom HillsMaterial to the world
pub struct HillsMaterialPlugin;

impl Plugin for HillsMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<HillsMaterial>::default());
    }
}

/// Custom material inspired by builtin [`ColorMaterial`]
#[derive(Debug, Clone, TypeUuid)]
#[uuid = "f781e582-4fd9-4a01-b870-1bb20bdb8c34"]
pub struct HillsMaterial {
    // Texture to blend into the background
    pub texture: Handle<Image>,
}

pub struct GpuHillsMaterial {
    bind_group: BindGroup,
}

impl RenderAsset for HillsMaterial {
    type ExtractedAsset = HillsMaterial;

    type PreparedAsset = GpuHillsMaterial;

    type Param = (
        SRes<RenderDevice>,
        SRes<Material2dPipeline<HillsMaterial>>,
        SRes<RenderAssets<Image>>,
    );

    fn extract_asset(&self) -> Self::ExtractedAsset {
        self.clone()
    }

    fn prepare_asset(
        material: Self::ExtractedAsset,
        (render_device, hills_pipeline, gpu_images): &mut bevy::ecs::system::SystemParamItem<
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
            label: Some("hills_material_bind_group"),
            layout: &hills_pipeline.material2d_layout,
        });

        Ok(GpuHillsMaterial { bind_group })
    }
}

// A SpecializedMaterial2d is used rather than the simpler Material2d, since the mesh uses a non-standard vertex buffer layout
impl SpecializedMaterial2d for HillsMaterial {
    fn bind_group(
        render_asset: &<Self as bevy::render::render_asset::RenderAsset>::PreparedAsset,
    ) -> &bevy::render::render_resource::BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(
        render_device: &bevy::render::renderer::RenderDevice,
    ) -> bevy::render::render_resource::BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("hills_material_layout"),
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
        Some(asset_server.load("shaders/hills.wgsl"))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load("shaders/hills.wgsl"))
    }

    type Key = ();
    fn key(_material: &<Self as RenderAsset>::PreparedAsset) -> Self::Key {}

    fn specialize(_key: Self::Key, descriptor: &mut RenderPipelineDescriptor) {
        let vertex_attributes = vec![VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        }];

        let vertex_array_stride = 8;

        descriptor.vertex.buffers = vec![VertexBufferLayout {
            array_stride: vertex_array_stride,
            step_mode: VertexStepMode::Vertex,
            attributes: vertex_attributes,
        }];
    }
}
