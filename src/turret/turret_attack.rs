use crate::physics::GameLayer;
use avian3d::prelude::*;
use bevy::prelude::*;

pub(super) struct TurretAttackPlugin;

impl Plugin for TurretAttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                turret_targeting,
                turret_shooting,
                handle_projectile_collisions,
                projectile_movement,
            )
                .chain(),
        )
        .add_observer(setup_enemy_collision);

        app.register_type::<Turret>()
            .register_type::<TurretCooldown>()
            .register_type::<PathPriority>()
            .register_type::<Enemy>()
            .register_type::<Health>();
    }
}

/// Add collision layers to enemies
fn setup_enemy_collision(
    trigger: Trigger<OnAdd, Enemy>,
    mut commands: Commands,
) {
    commands.entity(trigger.target()).insert((
        CollisionLayers::new(GameLayer::Enemy, LayerMask::ALL),
        CollisionEventsEnabled,
    ));
}

/// Find and target the best enemy
fn turret_targeting(
    mut commands: Commands,
    q_turrets: Query<(
        &GlobalTransform,
        &Turret,
        &CurrentTargets,
        Entity,
    )>,
    q_enemies: Query<
        (&GlobalTransform, &PathPriority, Entity),
        With<Enemy>,
    >,
    spatial_query: SpatialQuery,
) {
    for (turret_transform, turret, current_targets, turret_entity) in
        q_turrets.iter()
    {
        let turret_position = turret_transform.translation();

        // Find enemies in range using shape intersection
        let detection_sphere = Collider::sphere(turret.range);
        let intersections = spatial_query.shape_intersections(
            &detection_sphere,
            turret_position,
            Quat::IDENTITY,
            &SpatialQueryFilter::default()
                .with_mask(GameLayer::Enemy),
        );

        // Find best target from intersected entities
        let mut best_target = None;
        let mut best_priority = f32::MAX;

        for entity in intersections {
            let Ok((_enemy_transform, path_priority, enemy_entity)) =
                q_enemies.get(entity)
            else {
                continue;
            };

            // Check if this enemy has better priority
            if path_priority.0 < best_priority {
                best_priority = path_priority.0;
                best_target = Some(enemy_entity);
            }
        }

        let current_target = current_targets.first().copied();

        // Update target relationship
        match (current_target, best_target) {
            (Some(current), Some(best)) if current != best => {
                // Switch target by remove old and adding new
                commands.entity(current).remove::<TargetedBy>();
                commands
                    .entity(best)
                    .insert(TargetedBy(turret_entity));
            }
            (Some(current), None) => {
                // Lost target
                commands.entity(current).remove::<TargetedBy>();
            }
            (None, Some(best)) => {
                // New target
                commands
                    .entity(best)
                    .insert(TargetedBy(turret_entity));
            }
            _ => {
                // No change needed
            }
        }
    }
}

/// Shoot at current target
fn turret_shooting(
    mut commands: Commands,
    q_turrets: Query<(
        &GlobalTransform,
        &Turret,
        &CurrentTargets,
        Entity,
    )>,
    mut q_cooldowns: Query<&mut TurretCooldown>,
    q_enemies: Query<&GlobalTransform, With<Enemy>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    for (turret_transform, turret, current_targets, turret_entity) in
        q_turrets.iter()
    {
        let Ok(mut cooldown) = q_cooldowns.get_mut(turret_entity)
        else {
            continue;
        };

        cooldown.remaining -= time.delta_secs();
        if cooldown.remaining > 0.0 {
            continue;
        }

        // Check if there are any targets
        let Some(target_entity) = current_targets.first().copied()
        else {
            continue;
        };

        let Ok(target_transform) = q_enemies.get(target_entity)
        else {
            continue;
        };

        let turret_position = turret_transform.translation();
        let target_position = target_transform.translation();
        let projectile_start = turret_position + Vec3::Y * 0.5;
        let direction =
            (target_position - projectile_start).normalize();

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.8, 1.0),
                emissive: LinearRgba::rgb(0.5, 2.0, 3.0),
                ..default()
            })),
            Transform::from_translation(projectile_start),
            RigidBody::Kinematic,
            Collider::sphere(0.1),
            CollisionLayers::new(
                GameLayer::Projectile,
                GameLayer::Enemy,
            ),
            CollisionEventsEnabled,
            Projectile {
                velocity: direction * turret.projectile_speed,
                damage: turret.damage,
                lifetime: 3.0,
            },
            ProjectileFiredBy(turret_entity),
        ));

        cooldown.remaining = turret.attack_cooldown;
    }
}

/// Handle projectile collisions using physics system
fn handle_projectile_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionStarted>,
    q_projectiles: Query<&Projectile>,
    q_enemies: Query<(), With<Enemy>>,
    mut q_health: Query<&mut Health>,
) {
    for CollisionStarted(entity1, entity2) in collision_events.read()
    {
        // Check if one is projectile, other is enemy
        let (projectile_entity, enemy_entity) = if q_projectiles
            .contains(*entity1)
            && q_enemies.contains(*entity2)
        {
            (*entity1, *entity2)
        } else if q_projectiles.contains(*entity2)
            && q_enemies.contains(*entity1)
        {
            (*entity2, *entity1)
        } else {
            continue;
        };

        // Get projectile data and apply damage
        if let Ok(projectile) = q_projectiles.get(projectile_entity) {
            if let Ok(mut health) = q_health.get_mut(enemy_entity) {
                health.current -= projectile.damage;

                if health.current <= 0.0 {
                    commands.entity(enemy_entity).despawn();
                }
            }

            // Despawn projectile after hit
            commands.entity(projectile_entity).despawn();
        }
    }
}

/// Move projectiles
fn projectile_movement(
    mut commands: Commands,
    mut q_projectiles: Query<(
        &mut Transform,
        &mut Projectile,
        Entity,
    )>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for (mut transform, mut projectile, projectile_entity) in
        q_projectiles.iter_mut()
    {
        // Update lifetime
        projectile.lifetime -= delta_time;
        if projectile.lifetime <= 0.0 {
            commands.entity(projectile_entity).despawn();
            continue;
        }

        // Move projectile
        transform.translation += projectile.velocity * delta_time;
    }
}

/// Turret component with stats only
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(TurretCooldown, CurrentTargets)]
pub struct Turret {
    pub range: f32,
    pub damage: f32,
    pub attack_cooldown: f32,
    pub projectile_speed: f32,
}

/// Cooldown component for turrets
/// Tracks remaining time before the turret can fire again
#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct TurretCooldown {
    pub remaining: f32,
}

/// PathPriority for targeting (lower = higher priority)
// TODO: Will be changed to use a pathfinding algorithm
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct PathPriority(pub f32);

impl Default for PathPriority {
    fn default() -> Self {
        Self(f32::MAX)
    }
}

/// Enemy marker component
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(CollisionEventsEnabled, PathPriority)]
pub struct Enemy;

/// Health component for entities that can take damage
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

/// Projectile component representing a fired projectile
#[derive(Component, Debug)]
#[require(CollisionEventsEnabled)]
pub struct Projectile {
    pub velocity: Vec3,
    pub damage: f32,
    pub lifetime: f32,
}

/// Relationship components for turret targeting
#[derive(Component, Deref, Default, Debug)]
#[relationship_target(relationship = TargetedBy)]
pub struct CurrentTargets(Vec<Entity>);

#[derive(Component, Deref, Debug)]
#[relationship(relationship_target = CurrentTargets)]
pub struct TargetedBy(Entity);

/// Relationship components for projectiles
#[derive(Component, Deref, Default, Debug)]
#[relationship_target(relationship = ProjectileFiredBy)]
pub struct FiredProjectiles(Vec<Entity>);

#[derive(Component, Deref, Debug)]
#[relationship(relationship_target = FiredProjectiles)]
pub struct ProjectileFiredBy(Entity);
