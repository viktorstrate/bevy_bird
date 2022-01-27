use bevy::prelude::*;

use crate::{bevy_player::BevyPlayerPlugin, GameSystems};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(make_player);
        app.add_system(player_input.before(GameSystems::PlayerMovement));
        app.add_system(player_system.label(GameSystems::PlayerMovement));
        app.add_plugin(BevyPlayerPlugin);
    }
}

#[derive(Component)]
pub struct PlayerComponent {
    pub velocity: Vec2,
    pub diving: bool,
    pub bevy_mode: bool,
}

impl Default for PlayerComponent {
    fn default() -> Self {
        Self {
            velocity: Vec2::new(200., 0.),
            diving: false,
            bevy_mode: false,
        }
    }
}

fn make_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("textures/player.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 100., 1.),
            ..Default::default()
        })
        .insert(PlayerComponent::default());
}

fn ground_y(x: f32) -> f32 {
    return ((x / 256. * std::f32::consts::TAU + std::f32::consts::PI).sin() * 0.1 + 1.) * 256.
        - 256.;
}

fn ground_normal(x: f32) -> Vec2 {
    let derivative = (x / 256. * std::f32::consts::TAU + std::f32::consts::PI).cos();

    let normal = Vec2::new(-derivative, 1.).normalize();
    return normal;
}

fn player_system(mut player: Query<(&mut Transform, &mut PlayerComponent)>, time: Res<Time>) {
    let (mut transform, mut player) = player
        .get_single_mut()
        .expect("only one player component should exist");

    player.velocity.y -= if player.diving { 400. } else { 100. } * time.delta_seconds();

    let mut new_transform = transform.clone();

    new_transform.translation.y += player.velocity.y * time.delta_seconds();
    new_transform.translation.x += player.velocity.x * time.delta_seconds();

    let ground_normal = ground_normal(new_transform.translation.x);
    let ground_y = ground_y(new_transform.translation.x) + 16.;

    if new_transform.translation.y < ground_y {
        new_transform.translation.y = ground_y;

        let fwd = Vec2::new(ground_normal.y, -ground_normal.x);
        let mut new_velocity =
            fwd * fwd.dot(player.velocity.normalize()) * player.velocity.length();

        new_velocity.x = new_velocity.x.max(80.);

        player.velocity = new_velocity;
    }

    if new_transform.translation.y > 360. && player.velocity.y > 0. {
        player.velocity.y -= 300. * time.delta_seconds();
    }

    new_transform.rotation = Quat::from_euler(
        EulerRot::XYZ,
        0.,
        0.,
        player.velocity.y.atan2(player.velocity.x),
    );

    transform.translation = new_transform.translation;
    transform.rotation = new_transform.rotation;
}

fn player_input(
    mut player: Query<&mut PlayerComponent>,
    keys: Res<Input<KeyCode>>,
    touches: Res<Touches>,
) {
    let mut player = player
        .get_single_mut()
        .expect("only one player component should exist");

    player.diving = keys.pressed(KeyCode::Space) || touches.iter().next().is_some();

    if keys.just_pressed(KeyCode::B) {
        player.bevy_mode = !player.bevy_mode;
    }
}
