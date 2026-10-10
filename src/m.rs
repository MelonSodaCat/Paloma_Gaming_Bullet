
use bevy::{color::palettes::basic::TEAL, prelude::*};

///implificada sin enemigos
/// To Do:
/// Enemigos
/// Collision con Rapier
/// Enemy movement

const SPRITE_SIZE: f32 = 100.;
const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);
const PLAYER_SPEED: f32 = 240.0;

const PROJECTILE_SPEED: f32 = 500.0;
const PROJECTILE_LIFETIME: f32 = 2.0;
const FIRE_COOLDOWN: f32 = 0.1;
const PROJECTILE_SIZE: Vec2 = Vec2::new(20., 6.);
const PROJECTILE_DIRECTION: Vec2 = Vec2::Y;



fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_hz(60.0))//frame rate
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (move_player, despawn_expired, shoot, move_projectiles))
        .run();
}





///Player Component
/// Defines the player
#[derive(Component)]

struct Player;

///Movement Speed Component
/// that which moves
#[derive(Component)]

struct MovementSpeed(f32);

///Fire cooldown component
#[derive(Component)]
struct FireCooldown(Timer);

///Marks a projectile
#[derive(Component)]
struct Projectile;

//Projectile velocity (units per second)
#[derive(Component)]
struct Velocity(Vec2);
 
///Despawn timer so projectiles don't live forever
#[derive(Component)]
struct Lifetime(Timer);

//Marks an enemy
#[derive(Component)]
struct Enemy;
 
///Hit points left. At 0 the entity is despawned.
#[derive(Component)]
struct Health(i32);

 
///Shared mesh/material so every shot reuses the same handles
#[derive(Resource)]
struct ProjectileAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}



///Player Movement System
/// eso
/// pendiente: actualizar al modelo utilizado en Movement / Run physics in a fixed timestep
fn move_player(
    query: Single<(&Player, &mut Transform, &MovementSpeed)>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let mut direction = Vec2::ZERO;
    let (mc , mut transform, movement_speed) = query.into_inner();

    if kb_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    // Progressively update the player's position over time. Normalize the
    // direction vector to prevent it from exceeding a magnitude of 1 when
    // moving diagonally.
    let move_delta = direction.normalize_or_zero() * movement_speed.0 * time.delta_secs();
    transform.translation += move_delta.extend(0.);

      // Bound the ship within the invisible level bounds
    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);


}

///Moves every projectile along its velocity
fn move_projectiles(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity), With<Projectile>>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += (velocity.0 * time.delta_secs()).extend(0.);
    }
}

//Shoot System
/// Hold Space to fire a projectile in a straight line (PROJECTILE_DIRECTION)
/// from the player. Runs independently of move_player, so you can shoot while moving.
fn shoot(
    mut commands: Commands,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
    player: Single<(&Transform, &mut FireCooldown), With<Player>>,
    assets: Res<ProjectileAssets>,
) {
    let (player_transform, mut cooldown) = player.into_inner();
 
    cooldown.0.tick(time.delta());
 
    if !kb_input.pressed(KeyCode::Space) || !cooldown.0.is_finished() {
        return;
    }
 
    cooldown.0.reset();
 
    let origin = player_transform.translation.truncate();
    let direction = PROJECTILE_DIRECTION.normalize();
    let angle = direction.y.atan2(direction.x);
    commands.spawn((
        Mesh2d(assets.mesh.clone()),
        MeshMaterial2d(assets.material.clone()),
        Transform::from_translation((origin + direction * SPRITE_SIZE).extend(0.1))
            .with_rotation(Quat::from_rotation_z(angle))
            .with_scale(PROJECTILE_SIZE.extend(1.)),
        Projectile,
        Velocity(direction * PROJECTILE_SPEED),
        Lifetime(Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once)),
    ));
}



 
///Removes projectiles once their lifetime runs out
fn despawn_expired(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.0.tick(time.delta());
        if lifetime.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}





fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) { //2D Camera
    commands.spawn(Camera2d);
    //Minimal UI
    commands.spawn((
        Text::new("Use WASD to Move Around\n"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));

    commands.insert_resource(ProjectileAssets {
        mesh: meshes.add(Rectangle::default()),
        material: materials.add(Color::from(TEAL)),
    });

        //spawns the entity with those components
    //the bevy bird is a sprite, sized to SPRITE_SIZE (no scaling needed)
    commands.spawn((
        Sprite {
            image: asset_server.load("branding/icon.png"),
            custom_size: Some(Vec2::splat(SPRITE_SIZE)),
            ..default()
        },
        Transform::default(),
        Player,
        MovementSpeed(PLAYER_SPEED),
        FireCooldown(Timer::from_seconds(FIRE_COOLDOWN, TimerMode::Once)),
    ));

  
}


