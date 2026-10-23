use bevy::{color::palettes::basic::TEAL, prelude::*};

mod camera;


const SPRITE_SIZE: f32 = 100.;
const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);
const PLAYER_SPEED: f32 = 240.0;

const PROJECTILE_SPEED: f32 = 500.0;
const PROJECTILE_LIFETIME: f32 = 2.0;
const FIRE_COOLDOWN: f32 = 0.1;
const PROJECTILE_SIZE: Vec2 = Vec2::new(20., 6.);
const PROJECTILE_DIRECTION: Vec2 = Vec2::Y;
///Hit radius used for collisions (a bit smaller than the projectile's long side)
const PROJECTILE_RADIUS: f32 = 8.;

const ENEMY_SIZE: f32 = 40.;
///Hit radius used for collisions
const ENEMY_RADIUS: f32 = ENEMY_SIZE / 2.;
///How many shots an enemy survives
const ENEMY_HEALTH: i32 = 1;
const ENEMY_SPEED: f32 = 100.;
const ENEMY_COUNT: u32 = 5;
const ENEMY_SPACING: f32 = 120.;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins, 
            camera::CameraPlugin
            ))
        .insert_resource(Time::<Fixed>::from_hz(60.0)) //frame rate
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                move_player,
                shoot,
                apply_velocity,
                projectile_hits_enemies,
                despawn_expired,
            )
                .chain(), //run in this exact order every tick
        )
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

///Marks an enemy
#[derive(Component)]
struct Enemy;

///Hit points left. At 0 the entity is despawned.
#[derive(Component)]
struct Health(i32);

//Velocity (units per second). Used by projectiles AND enemies.
#[derive(Component)]
struct Velocity(Vec2);

///Despawn timer so projectiles don't live forever
#[derive(Component)]
struct Lifetime(Timer);

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
    let (_mc, mut transform, movement_speed) = query.into_inner();

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

///Moves everything that has a Velocity projectiles
fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity), With<Projectile>>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += (velocity.0 * time.delta_secs()).extend(0.);
    }
}



///Collision System
/// Every tick, check each projectile against each enemy. If they overlap, the projectile
/// is removed and the enemy loses 1 health; at 0 health the enemy is removed.
/// "Overlap" here is: distance between centers < sum of both radii.
fn projectile_hits_enemies(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
    mut enemies: Query<(Entity, &Transform, &mut Health), With<Enemy>>,
) {
    for (projectile_entity, projectile_transform) in &projectiles {
        let projectile_pos = projectile_transform.translation.truncate();

        for (enemy_entity, enemy_transform, mut health) in &mut enemies {
            let enemy_pos = enemy_transform.translation.truncate();

            if projectile_pos.distance(enemy_pos) < ENEMY_RADIUS + PROJECTILE_RADIUS {
                commands.entity(projectile_entity).despawn();

                health.0 -= 1;
                // == 0 (not <= 0) so two shots landing on the same tick can't despawn it twice
                if health.0 == 0 {
                    commands.entity(enemy_entity).despawn();
                }

                // this projectile is used up, don't check it against other enemies
                break;
            }
        }
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
) {

    //Minimal UI
    commands.spawn((
        Text::new("Use WASD to Move Around\nSpace to Shoot"),
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

    //spawns a row of enemies near the top, moving sideways together
    for i in 0..ENEMY_COUNT {
        let x = (i as f32 - (ENEMY_COUNT as f32 - 1.) / 2.) * ENEMY_SPACING;
        commands.spawn((
            Sprite::from_color(Color::srgb(0.85, 0.25, 0.3), Vec2::splat(ENEMY_SIZE)),
            Transform::from_xyz(x, BOUNDS.y / 2. - 60., 0.),
            Enemy,
            Health(ENEMY_HEALTH),
            Velocity(Vec2::X * ENEMY_SPEED),
        ));
    }

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