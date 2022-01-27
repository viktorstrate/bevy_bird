#![feature(let_else)]

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use hills::HillsMaterial;

mod background;
mod bevy_player;
mod hills;
mod player;

pub fn start_game() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(hills::HillsMaterialPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(background::BackgroundPlugin)
        .add_startup_system(setup_world)
        .add_startup_system(asset_server_changes)
        .add_system(hills_system.after(GameSystems::Camera))
        .add_system(
            camera_movement_system
                .label(GameSystems::Camera)
                .after(GameSystems::PlayerMovement),
        )
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(1., 1., 1.)))
        // debug
        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .run();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemLabel)]
pub enum GameSystems {
    PlayerMovement,
    Camera,
}

fn asset_server_changes(asset_server: ResMut<AssetServer>) {
    asset_server.watch_for_changes().unwrap();
}

fn setup_world(mut commands: Commands) {
    // Spawn camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

#[derive(Component)]
struct HillComponent;

// Add the hills object to the world
fn hills_system(
    mut commands: Commands,
    mut assets: (ResMut<Assets<Mesh>>, ResMut<Assets<HillsMaterial>>),
    asset_server: ResMut<AssetServer>,
    hills: Query<(&Transform, Entity), With<HillComponent>>,
    cameras: Query<&Transform, With<Camera>>,
    windows: Res<Windows>,
) {
    let camera_trans = cameras.get_single().unwrap();
    let window = windows.get_primary().unwrap();

    let mut first_hill: Option<(&Transform, Entity)> = None;
    let mut last_hill: Option<(&Transform, Entity)> = None;

    for hill in hills.iter() {
        if let Some(first) = first_hill {
            if first.0.translation.x > hill.0.translation.x {
                first_hill = Some(hill);
            }
        } else {
            first_hill = Some(hill);
        }

        if let Some(last) = last_hill {
            if last.0.translation.x < hill.0.translation.x {
                last_hill = Some(hill);
            }
        } else {
            last_hill = Some(hill);
        }
    }

    if let Some((&last_trans, _)) = last_hill {
        // Spawn new hills to the right
        if camera_trans.translation.x > last_trans.translation.x - window.width() {
            spawn_hill(
                Transform::default()
                    .with_scale(Vec3::splat(256.))
                    .with_translation(Vec3::new(last_trans.translation.x + 256., -256., 0.)),
                &mut commands,
                &mut assets,
                &asset_server,
            );
        }
    } else {
        // If no hills were found, spawn initial ones
        for i in 0..(window.width() as i32 / 256 * 2) {
            spawn_hill(
                Transform::default()
                    .with_scale(Vec3::splat(256.))
                    .with_translation(Vec3::new((i as f32) * 256. - window.width(), -256., 0.)),
                &mut commands,
                &mut assets,
                &asset_server,
            );
        }
    }

    // Despawn hills to the left
    if let Some((&first_trans, first_entity)) = first_hill {
        if camera_trans.translation.x > first_trans.translation.x + window.width() {
            commands.entity(first_entity).despawn();
        }
    }
}

fn spawn_hill(
    transform: Transform,
    commands: &mut Commands,
    (meshes, materials): &mut (ResMut<Assets<Mesh>>, ResMut<Assets<HillsMaterial>>),
    asset_server: &ResMut<AssetServer>,
) {
    // Make a new custom HillsMaterial to use with the mesh.
    // This material also specifies the structure of the vertices (vec2 for position and no normal or uv maps)
    let hills_material = HillsMaterial {
        texture: asset_server.load("textures/paper-seamless.png"),
    };

    // Add the mesh to the world
    commands
        .spawn_bundle(MaterialMesh2dBundle {
            mesh: meshes.add(hills::hills_mesh()).into(),
            material: materials.add(hills_material),
            transform,
            ..Default::default()
        })
        .insert(HillComponent);
}

fn camera_movement_system(
    mut cameras: Query<&mut Transform, With<Camera>>,
    player: Query<(&Transform, &player::PlayerComponent), Without<Camera>>,
    time: Res<Time>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();

    let (player_trans, player) = player
        .get_single()
        .expect("only one player component should exist");

    for mut cam in cameras.iter_mut() {
        let desired_scale = ((player.velocity.x + 80.) / 400. + 0.2).min(1.6);

        let dist = time.delta_seconds().min(1.);
        let new_scale = cam.scale.x * (1. - dist) + desired_scale * dist;

        cam.scale.x = new_scale;
        cam.scale.y = new_scale;

        cam.translation.x = player_trans.translation.x;
        cam.translation.y = player_trans
            .translation
            .y
            .min(256. / new_scale)
            .max((window.height() / 2. - 256. / new_scale) * new_scale);
    }
}
