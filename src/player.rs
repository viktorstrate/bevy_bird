use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(make_player);
        app.add_system(player_input);
        app.add_system(player_system);
    }
}

#[derive(Component)]
pub struct PlayerComponent {
    velocity: Vec2,
    diving: bool,
}

impl Default for PlayerComponent {
    fn default() -> Self {
        Self {
            velocity: Vec2::new(200., 0.),
            diving: false,
        }
    }
}

fn make_player(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
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
    for (mut transform, mut player) in player.iter_mut() {
        player.velocity.y -= if player.diving { 250. } else { 50. } * time.delta_seconds();

        transform.translation.y += player.velocity.y * time.delta_seconds();
        transform.translation.x += player.velocity.x * time.delta_seconds();

        let ground_normal = ground_normal(transform.translation.x);
        let ground_y = ground_y(transform.translation.x) + 25.;

        if transform.translation.y < ground_y {
            transform.translation.y = ground_y;

            let fwd = Vec2::new(ground_normal.y, -ground_normal.x);
            let mut new_velocity =
                fwd * fwd.dot(player.velocity.normalize()) * player.velocity.length();

            new_velocity.x = new_velocity.x.max(80.);

            player.velocity = new_velocity;
        }

        if transform.translation.y > 360. && player.velocity.y > 0. {
            player.velocity.y -= 300. * time.delta_seconds();
        }

        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            0.,
            0.,
            player.velocity.y.atan2(player.velocity.x),
        );
    }
}

fn player_input(keys: Res<Input<KeyCode>>, mut player: Query<&mut PlayerComponent>) {
    for mut player in player.iter_mut() {
        player.diving = keys.pressed(KeyCode::Space);
    }
}
