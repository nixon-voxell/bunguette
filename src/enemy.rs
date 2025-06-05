use bevy::prelude::*;

use crate::tile::TileMap;

pub(super) struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            pathfind.after(TransformSystem::TransformPropagate),
        );

        app.register_type::<FinalTarget>().register_type::<Enemy>();
    }
}

fn pathfind(
    mut q_enemies: Query<(&mut EnemyPath, &GlobalTransform, Entity)>,
    q_final_target: Query<&GlobalTransform, With<FinalTarget>>,
    tile_map: Res<TileMap>,
) {
    let Ok(final_target) = q_final_target.single() else {
        return;
    };

    for (mut enemy_path, transform, entity) in q_enemies.iter_mut() {
        let start_translation = transform.translation();
        let end_translation = final_target.translation();

        // Pathfind if it's just newly added or the tile map
        // has been updated.
        if enemy_path.is_empty() || tile_map.is_changed() {
            info!("pathfind: {start_translation}, {end_translation}");
            if let Some(path_to_target) = tile_map.pathfind_to(
                &start_translation,
                &end_translation,
                false,
            ) {
                println!("{:#?}", path_to_target);
                enemy_path.0 = path_to_target;
            } else if let Some(path_to_tower) = tile_map.pathfind_to(
                &start_translation,
                &end_translation,
                true,
            ) {
                enemy_path.0 = path_to_tower;
            } else {
                info!("Can't find path for enemy {entity}!");
            }
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FinalTarget;

/// Tag component for enemy units.
#[derive(Component, Reflect)]
#[require(EnemyPath)]
#[reflect(Component)]
pub struct Enemy;

/// The current path of the enemy.
#[derive(Component, Deref, Default)]
pub struct EnemyPath(Vec<IVec2>);
