use bevy::{prelude::*, utils::HashMap};

use crate::{player::PlayerComponent, GameSystems};

/// Pressing B will toggle the player to show a Bevy bird rather than the normal Tiny Wings bird

pub struct BevyPlayerPlugin;

impl Plugin for BevyPlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(make_bevy_player);
        app.add_system(bevy_player_system.after(GameSystems::PlayerMovement));
        app.add_system(bevy_mode_visibility_system);
    }
}

#[derive(Component)]
pub struct BevyPlayerComponent {
    parent: Option<Entity>,
    previous_transform: Transform,
}

fn make_bevy_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut parent = commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("textures/bevy.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 36.0)),
                color: Color::hsl(0., 0., 0.95),
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 100., 1.),
            ..Default::default()
        })
        .insert(BevyPlayerComponent {
            parent: None,
            previous_transform: Transform::default(),
        })
        .id();

    const TRAIL_COUNT: i32 = 6;
    for i in 0..(TRAIL_COUNT) {
        parent = commands
            .spawn_bundle(SpriteBundle {
                texture: asset_server.load("textures/bevy.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(50.0, 36.0)),
                    color: Color::hsla(0., 0., 0.95 - (i as f32 * 0.1), 0.8 - (i as f32 * 0.1)),
                    ..Default::default()
                },
                transform: Transform::from_xyz(0., 100., 1. - (i as f32 * 0.01)),
                ..Default::default()
            })
            .insert(BevyPlayerComponent {
                parent: Some(parent),
                previous_transform: Transform::default(),
            })
            .id();
    }
}

fn bevy_player_system(
    player: Query<(&Transform, &PlayerComponent)>,
    mut bevy_players: Query<(&mut Transform, &mut BevyPlayerComponent), Without<PlayerComponent>>,
) {
    let (player_trans, player) = player
        .get_single()
        .expect("only one player component should exist");

    if !player.bevy_mode {
        for (mut bevy_player_trans, mut bevy_player) in bevy_players.iter_mut() {
            bevy_player.previous_transform = Transform::default();
            bevy_player_trans.translation.x = 0.;
            bevy_player_trans.translation.y = -1000.;
        }
        return;
    }

    for (mut bevy_player_trans, mut bevy_player) in bevy_players.iter_mut() {
        bevy_player.previous_transform = bevy_player_trans.clone();

        if bevy_player.parent.is_none() {
            bevy_player_trans.translation = player_trans.translation;
            bevy_player_trans.rotation = player_trans.rotation;
        }
    }

    let mut parent_trans_lookup: HashMap<Entity, Transform> = HashMap::default();

    for (_, bevy_player) in bevy_players.iter() {
        if let Some(parent_entity) = bevy_player.parent {
            let (parent_trans, _) = bevy_players
                .get(parent_entity)
                .expect("Bevy player parent should exist");

            parent_trans_lookup.insert(parent_entity, parent_trans.clone());
        }
    }

    for (mut bevy_player_trans, bevy_player) in bevy_players.iter_mut() {
        if let Some(parent_entity) = bevy_player.parent {
            let parent_transform = parent_trans_lookup
                .get(&parent_entity)
                .expect("HashMap should contain parent");

            bevy_player_trans.translation.x = parent_transform.translation.x;
            bevy_player_trans.translation.y = parent_transform.translation.y;
            bevy_player_trans.rotation = parent_transform.rotation;
        }
    }
}

fn bevy_mode_visibility_system(
    mut player: Query<(&mut Visibility, &PlayerComponent)>,
    mut bevy_players: Query<(
        &mut Visibility,
        (With<BevyPlayerComponent>, Without<PlayerComponent>),
    )>,
) {
    let (mut player_visibility, player) = player
        .get_single_mut()
        .expect("only one player component should exist");

    player_visibility.is_visible = !player.bevy_mode;

    for (mut visibility, _) in bevy_players.iter_mut() {
        visibility.is_visible = player.bevy_mode;
    }
}
