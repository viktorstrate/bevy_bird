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
    sprite::{Material2dPipeline, Material2dPlugin, MaterialMesh2dBundle, SpecializedMaterial2d},
};

/// This example shows how to manually render 2d items using "mid level render apis" with a custom pipeline for 2d meshes
/// It doesn't use the [`Material2d`] abstraction, but changes the vertex buffer to include vertex color
/// Check out the "mesh2d" example for simpler / higher level 2d meshes
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(HillsMaterialPlugin)
        .add_startup_system(spawn_hills)
        .add_startup_system(asset_server_changes)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(1., 1., 1.)))
        .run();
}

fn asset_server_changes(asset_server: ResMut<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    let _ = asset_server.load::<Shader, _>("shaders/hills.wgsl");
}

// Add the hills object to the world
fn spawn_hills(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<HillsMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    // Generate vertex positions
    const STEPS: i32 = 100;
    let mut v_pos = vec![];

    for i in 0..STEPS {
        let x_offset = (i as f32) / (STEPS as f32) - 0.5;
        v_pos.push([x_offset, 0.]);
        v_pos.push([
            x_offset,
            (i as f32 / (STEPS as f32) * std::f32::consts::TAU).sin() * 0.2 + 1.,
        ]);
    }

    // Generate indices for vertex positions
    let mut indices = vec![];
    for i in 0..(STEPS - 1) {
        let x = (i * 2) as u32;
        indices.extend_from_slice(&[x, x + 3, x + 1]);
        indices.extend_from_slice(&[x, x + 2, x + 3]);
    }

    // Save vertex data to a new mesh
    let mut hills_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    hills_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    hills_mesh.set_indices(Some(Indices::U32(indices)));

    // Make a new custom HillsMaterial to use with the mesh.
    // This material also specifies the structure of the vertices (vec2 for position and no normal or uv maps)
    let hills_material = HillsMaterial {
        texture: asset_server.load("textures/paper-seamless.png"),
    };

    // Add the mesh to the world
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(hills_mesh).into(),
        material: materials.add(hills_material),
        transform: Transform::default()
            .with_scale(Vec3::splat(256.))
            .with_translation(Vec3::new(0., -256., 0.)),
        ..Default::default()
    });

    // Spawn camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

// Plugin to register the custom HillsMaterial to the world
struct HillsMaterialPlugin;

impl Plugin for HillsMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<HillsMaterial>::default());
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "f781e582-4fd9-4a01-b870-1bb20bdb8c34"]
struct HillsMaterial {
    // Texture to blend into the background
    texture: Handle<Image>,
}

/// Custom material inspired by builtin [`ColorMaterial`]
struct GpuHillsMaterial {
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
        Some(asset_server.load::<Shader, _>("shaders/hills.wgsl"))
    }

    fn fragment_shader(asset_server: &AssetServer) -> Option<Handle<Shader>> {
        Some(asset_server.load::<Shader, _>("shaders/hills.wgsl"))
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
