//! The Bevy shell that draws the engine's world and forwards input.
//!
//! This binary is intentionally thin. It owns **no game rules**: it asks the
//! [`engine`] crate's [`World`] what to draw and where the player stands, and
//! it hands keyboard input back to [`World::try_move`]. All "can I walk there?"
//! logic lives in `engine`, which is why that crate has real unit tests and
//! this file needs none.
//!
//! A quick map of the Bevy concepts used here (for learners):
//! - [`App`] — the whole program; plugins and systems attach to it.
//! - [`Component`] — data attached to an entity (here: the `Player` marker).
//! - [`Resource`] — app-wide singleton data (the engine world + player id).
//! - *System* — a function that runs on a schedule (setup once, movement each frame).
//! - [`Query`] — how a system asks Bevy for the components it needs.

use bevy::prelude::*;
use engine::{Entity, EntityId, Map, PointDelta, Position, TileKind, World};

/// Edge length of one tile in world units (the camera scales pixels).
const TILE: f32 = 32.0;

/// Marker component on the player's sprite.
#[derive(Component)]
struct Player;

/// The engine's `EntityId` for the player, kept where systems can read it.
#[derive(Resource)]
struct PlayerId(EntityId);

/// Wraps the engine [`World`] so Bevy can store it as a [`Resource`].
#[derive(Resource)]
struct WorldResource(World);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_movement)
        .run();
}

/// Builds the engine world once, then spawns one sprite per tile plus the player.
fn setup(mut commands: Commands) {
    // --- 1. The engine world is the source of truth. ---
    let map = Map::bordered(21, 13).expect("map size is valid");
    let mut world = World::new(map);

    let player_id = EntityId(1);
    let player = Entity::new(player_id, "Wanderer", Position::new(10, 6)).expect("entity is valid");
    world.spawn(player).expect("spawn position is walkable");

    // --- 2. One sprite per tile, colored by terrain. ---
    for point in world.map().points() {
        let is_wall = world.map().tile_at(point) == Some(TileKind::Wall);
        let color = if is_wall {
            Color::srgb(0.48, 0.44, 0.38)
        } else {
            Color::srgb(0.16, 0.17, 0.20)
        };
        let (x, y) = Position::from(point).to_world_xy(TILE);
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(TILE)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.0),
        ));
    }

    // --- 3. The player: a smaller, brighter square on its tile. ---
    let player_position = world.entity(player_id).expect("player exists").position();
    let (px, py) = player_position.to_world_xy(TILE);
    commands.spawn((
        Sprite {
            color: Color::srgb(0.25, 0.85, 0.45),
            custom_size: Some(Vec2::splat(TILE * 0.7)),
            ..default()
        },
        Transform::from_xyz(px, py, 1.0),
        Player,
    ));

    // --- 4. Point the camera at the middle of the map. ---
    let center = Position::new(
        (world.map().width() / 2) as i32,
        (world.map().height() / 2) as i32,
    );
    let (cx, cy) = center.to_world_xy(TILE);
    commands.spawn((Camera2d, Transform::from_xyz(cx, cy, 0.0)));

    // --- 5. Make the world and player id available to other systems. ---
    commands.insert_resource(WorldResource(world));
    commands.insert_resource(PlayerId(player_id));
}

/// Reads the arrow/WASD keys once per press and asks the engine to move.
fn handle_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut world: ResMut<WorldResource>,
    player: Res<PlayerId>,
    mut player_sprite: Query<&mut Transform, With<Player>>,
) {
    let delta = if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        Some(PointDelta::NORTH)
    } else if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        Some(PointDelta::SOUTH)
    } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        Some(PointDelta::WEST)
    } else if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        Some(PointDelta::EAST)
    } else {
        None
    };

    // If no movement key was pressed this frame, do nothing.
    let Some(delta) = delta else { return };

    // Ask the engine. If it succeeds, sync the sprite to the new cell.
    if world.0.try_move(player.0, delta).is_ok() {
        let position = world.0.entity(player.0).expect("player exists").position();
        let (x, y) = position.to_world_xy(TILE);
        if let Ok(mut transform) = player_sprite.single_mut() {
            transform.translation.x = x;
            transform.translation.y = y;
        }
    }
}
