use bevy::prelude::*;

const PLAYER_SPEED: f32 = 240.0;
const SPRITE_SIZE: f32 = 100.;

///Player Component
/// Defines the player
#[derive(Component)]
struct Player;

///Movement Speed Component
/// that which moves
#[derive(Component)]
struct MovementSpeed(f32);

pub struct PlayerPlugin;

