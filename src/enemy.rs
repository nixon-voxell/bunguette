use avian3d::prelude::*;
use bevy::prelude::*;

use crate::tile::TileMap;

pub(super) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            pathfind.after(TransformSystem::TransformPropagate),
        )
        .add_systems(FixedUpdate, enemy_movement)
        .add_observer(on_path_changed);

        app.register_type::<FinalTarget>().register_type::<Enemy>();
    }
}

fn pathfind(
    mut commands: Commands,
    q_enemies: Query<(&Path, &GlobalTransform, Entity)>,
    q_final_target: Query<&GlobalTransform, With<FinalTarget>>,
    tile_map: Res<TileMap>,
) {
    let Ok(final_target) = q_final_target.single() else {
        return;
    };

    for (enemy_path, transform, entity) in q_enemies.iter() {
        let start_translation = transform.translation();
        let end_translation = final_target.translation();

        // Pathfind if it's just newly added or the tile map has been updated.
        if enemy_path.is_empty() || tile_map.is_changed() {
            info!("pathfind: {start_translation}, {end_translation}");
            if let Some(path_to_final) = tile_map.pathfind_to(
                &start_translation,
                &end_translation,
                false,
            ) {
                info!("To target: {:?}", path_to_final);
                commands
                    .entity(entity)
                    .insert((Path(path_to_final), TargetType::Final));
            } else if let Some(path_to_tower) = tile_map.pathfind_to(
                &start_translation,
                &end_translation,
                true,
            ) {
                info!("To tower: {:?}", path_to_tower);
                commands
                    .entity(entity)
                    .insert((Path(path_to_tower), TargetType::Tower));
            } else {
                warn!("Can't find path for enemy {entity}!");
            }
        }
    }
}

fn on_path_changed(
    trigger: Trigger<OnInsert, Path>,
    mut commands: Commands,
) {
    commands
        .entity(trigger.target())
        .insert(PathIndex(0))
        .remove::<TargetReached>();
}

fn enemy_movement(
    mut commands: Commands,
    mut q_enemies: Query<
        (
            &Enemy,
            &Path,
            &mut PathIndex,
            &mut LinearVelocity,
            &Position,
            Entity,
        ),
        Without<TargetReached>,
    >,
) {
    for (
        enemy,
        path,
        mut path_index,
        mut linear_velocity,
        position,
        entity,
    ) in q_enemies.iter_mut()
    {
        let Some(target_position) = path.get_target(&path_index)
        else {
            linear_velocity.0 = Vec3::ZERO;
            commands.entity(entity).insert(TargetReached);
            continue;
        };

        let current_position = position.xz();

        if current_position.distance(target_position) < 0.1 {
            path_index.increment();
        }

        let target_velocity = (target_position - current_position)
            .normalize()
            * enemy.movement_speed;

        linear_velocity.0 =
            Vec3::new(target_velocity.x, 0.0, target_velocity.y);
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FinalTarget;

/// Tag component for enemy units.
#[derive(Component, Reflect)]
#[require(Path, CollisionEventsEnabled)]
#[reflect(Component)]
pub struct Enemy {
    movement_speed: f32,
}

/// The current path of the enemy.
#[derive(Component, Deref, Default)]
#[require(PathIndex)]
#[component(immutable)]
pub struct Path(Vec<Vec2>);

impl Path {
    pub fn get_target(&self, index: &PathIndex) -> Option<Vec2> {
        self.0.get(index.0).copied()
    }
}

#[derive(Component, Deref, Default)]
pub struct PathIndex(usize);

impl PathIndex {
    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub enum TargetType {
    Tower,
    Final,
}

#[derive(Component)]
pub struct TargetReached;
