use bevy::{
    ecs::system::lifetimeless::SRes,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::Indices,
        render_asset::{RenderAsset, RenderAssets},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor, PrimitiveTopology,
            RenderPipelineDescriptor, VertexAttribute, VertexBufferLayout, VertexFormat,
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
        .insert_resource(ClearColor(Color::rgb(1., 1., 1.)))
        .run();
}

fn asset_server_changes(asset_server: ResMut<AssetServer>) {
    asset_server.watch_for_changes().unwrap();

    let _ = asset_server.load::<Shader, _>("shaders/hills.wgsl");
}

fn spawn_hills(
    mut commands: Commands,
    // We will add a new Mesh for the star being created
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<HillsMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let mut hills_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    const STEPS: i32 = 50;

    let mut v_pos = vec![];

    for i in 0..STEPS {
        let x_offset = (i as f32) / (STEPS as f32) - 0.5;
        v_pos.push([x_offset, 0.]);
        v_pos.push([
            x_offset,
            (i as f32 / (STEPS as f32) * std::f32::consts::TAU).sin() * 0.2 + 1.,
        ]);
    }

    // Set the position attribute
    hills_mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);

    let mut indices = vec![];
    for i in 0..(STEPS - 1) {
        let x = (i * 2) as u32;
        indices.extend_from_slice(&[x, x + 3, x + 1]);
        indices.extend_from_slice(&[x, x + 2, x + 3]);
    }

    // indices = vec![0, 2, 1, 0, 3, 2];
    hills_mesh.set_indices(Some(Indices::U32(indices)));

    let hills_material = HillsMaterial {
        texture: asset_server.load("textures/paper-seamless.png"),
    };

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(hills_mesh).into(),
        // mesh: meshes.add(mesh_quad).into(),
        transform: Transform::default().with_scale(Vec3::splat(128.)),
        material: materials.add(hills_material),
        ..Default::default()
    });

    // Spawn camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

struct HillsMaterialPlugin;

impl Plugin for HillsMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<HillsMaterial>::default());
    }
}

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "f781e582-4fd9-4a01-b870-1bb20bdb8c34"]
struct HillsMaterial {
    texture: Handle<Image>,
}

struct GpuHillsMaterial {
    // buffer: Buffer,
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
        extracted_asset: Self::ExtractedAsset,
        (render_device, hills_pipeline, gpu_images): &mut bevy::ecs::system::SystemParamItem<
            Self::Param,
        >,
    ) -> Result<
        Self::PreparedAsset,
        bevy::render::render_asset::PrepareAssetError<Self::ExtractedAsset>,
    > {
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[
                // BindGroupEntry {
                //     binding: 0,
                //     resource: buffer.as_entire_binding(),
                // },
                // BindGroupEntry {
                //     binding: 1,
                //     resource: BindingResource::TextureView(texture_view),
                // },
                // BindGroupEntry {
                //     binding: 2,
                //     resource: BindingResource::Sampler(sampler),
                // },
            ],
            label: Some("hills_material_bind_group"),
            layout: &hills_pipeline.material2d_layout,
        });

        Ok(GpuHillsMaterial { bind_group })
    }
}

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
            entries: &[],
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
