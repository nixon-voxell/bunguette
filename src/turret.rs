use avian3d::prelude::*;
use bevy::{color::palettes::css::*, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::action::{PlayerAction, TargetAction};
use crate::camera_controller::CameraTarget;
use crate::character_controller::CharacterController;
use crate::inventory::item::{ItemRegistry, ItemType};
use crate::inventory::{Inventory, Item};
use crate::player::PlayerType;

pub struct TurretPlacementPlugin;

impl Plugin for TurretPlacementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_placement_mode,
                show_placement_preview,
                place_turret,
            )
                .chain(),
        );
        app.register_type::<PlacementTile>();
    }
}

fn handle_placement_mode(
    mut commands: Commands,
    mut q_players: Query<
        (Entity, &Inventory, &TargetAction, Option<&InPlacementMode>),
        With<CharacterController>,
    >,
    q_actions: Query<&ActionState<PlayerAction>>,
    q_previews: Query<Entity, With<Preview>>,
) {
    for (player_entity, inventory, target_action, placement_mode) in
        q_players.iter_mut()
    {
        let Ok(action_state) = q_actions.get(target_action.get())
        else {
            continue;
        };

        // TODO: Change to actual player action
        if action_state
            .just_pressed(&PlayerAction::EnterPlacementMode)
        {
            if placement_mode.is_some() {
                commands
                    .entity(player_entity)
                    .remove::<InPlacementMode>();
                for preview in q_previews.iter() {
                    commands.entity(preview).despawn();
                }
            } else if let Some(selected_tower) =
                inventory.selected_tower.clone()
            {
                if inventory
                    .towers()
                    .get(&selected_tower)
                    .copied()
                    .unwrap_or(0)
                    > 0
                {
                    commands
                        .entity(player_entity)
                        .insert(InPlacementMode);
                }
            }
        }
    }
}

fn show_placement_preview(
    mut commands: Commands,
    // Find players in placement mode
    q_placement_players: Query<
        (Entity, &PlayerType),
        (With<CharacterController>, With<InPlacementMode>),
    >,
    // Find camera targets
    q_camera_targets: Query<
        (Entity, &GlobalTransform, &PlayerType),
        With<CameraTarget>,
    >,
    q_tiles: Query<
        (Entity, &GlobalTransform),
        (With<PlacementTile>, Without<TileOccupied>),
    >,
    q_previews: Query<Entity, With<Preview>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    spatial_query: SpatialQuery,
) {
    for preview in q_previews.iter() {
        commands.entity(preview).despawn();
    }

    // Only process players who are actually in placement mode
    for (placement_entity, placement_player_type) in
        q_placement_players.iter()
    {
        // Find the camera target for this specific player
        let camera_target = q_camera_targets.iter().find(
            |(target_entity, _, target_player_type)| {
                // Check if it's the same entity OR same player type
                *target_entity == placement_entity
                    || *target_player_type == placement_player_type
            },
        );

        if let Some((_, target_global_transform, _)) = camera_target {
            let target_position =
                target_global_transform.translation();

            // Create a sphere collider for intersection testing around the camera target
            let interaction_sphere = Collider::sphere(6.0);

            // Find tiles that intersect with the camera target's interaction sphere
            let mut valid_tiles = Vec::new();

            for (entity, tile_transform) in q_tiles.iter() {
                let tile_position = tile_transform.translation();

                // Check if the tile intersects with the interaction sphere
                let intersections = spatial_query
                    .shape_intersections(
                        &interaction_sphere,
                        target_position,
                        Quat::IDENTITY,
                        &SpatialQueryFilter::default(),
                    );

                // Check if this tile entity is in the intersections
                if intersections.iter().any(|&intersected_entity| {
                    intersected_entity == entity
                }) {
                    let distance_square = target_position
                        .distance_squared(tile_position);
                    valid_tiles.push((
                        entity,
                        tile_position,
                        distance_square,
                    ));
                }
            }

            // Find the closest valid tile
            if let Some((_, closest_tile_position, distance)) =
                valid_tiles.into_iter().min_by(
                    |(_, _, distance_a), (_, _, distance_b)| {
                        distance_a.partial_cmp(distance_b).unwrap()
                    },
                )
            {
                println!(
                    "Preview for {:?} at tile: {:?}, distance: {:.2}",
                    placement_player_type,
                    closest_tile_position,
                    distance.sqrt()
                );

                // Spawn a preview cube at the tile position
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(1.5, 1.5, 1.5))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgba(0.0, 1.0, 0.0, 0.8),
                        alpha_mode: AlphaMode::Blend,
                        emissive: LinearRgba::rgb(0.0, 2.0, 0.0),
                        ..default()
                    })),
                    Transform::from_translation(
                        closest_tile_position + Vec3::Y * 1.0,
                    ),
                    Preview,
                ));
            }
        }
    }
}

/// Places a turret on the closest available tile
fn place_turret(
    mut commands: Commands,
    mut q_placement_players: Query<
        (Entity, &mut Inventory, &TargetAction, &PlayerType),
        (With<CharacterController>, With<InPlacementMode>),
    >,
    q_actions: Query<&ActionState<PlayerAction>>,
    q_camera_targets: Query<
        (Entity, &GlobalTransform, &PlayerType),
        With<CameraTarget>,
    >,
    q_tiles: Query<
        (Entity, &GlobalTransform),
        (With<PlacementTile>, Without<TileOccupied>),
    >,
    q_previews: Query<Entity, With<Preview>>,
    item_registry: ItemRegistry,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    spatial_query: SpatialQuery,
) {
    for (player_entity, mut inventory, target_action, player_type) in
        q_placement_players.iter_mut()
    {
        let Ok(action_state) = q_actions.get(target_action.get())
        else {
            continue;
        };
        if !action_state.just_pressed(&PlayerAction::PlaceTurret) {
            continue;
        }
        let Some(selected_tower) = inventory.selected_tower.clone()
        else {
            continue;
        };

        let camera_target = q_camera_targets.iter().find(
            |(target_entity, _, target_player_type)| {
                *target_entity == player_entity
                    || **target_player_type == *player_type
            },
        );

        let Some((_, target_global_transform, _)) = camera_target
        else {
            continue;
        };
        let target_position = target_global_transform.translation();

        // Create a sphere collider for intersection
        let interaction_sphere = Collider::sphere(6.0);

        // Find tiles that intersect with the camera target's interaction sphere
        let mut valid_tiles = Vec::new();

        for (entity, tile_transform) in q_tiles.iter() {
            let tile_position = tile_transform.translation();

            // Check if the tile intersects with the interaction sphere
            let intersections = spatial_query.shape_intersections(
                &interaction_sphere,
                target_position,
                Quat::IDENTITY,
                &SpatialQueryFilter::default(),
            );

            // Check if this tile entity is in the intersections
            if intersections.iter().any(|&intersected_entity| {
                intersected_entity == entity
            }) {
                let distance_square =
                    target_position.distance_squared(tile_position);
                valid_tiles.push((
                    entity,
                    tile_position,
                    distance_square,
                ));
            }
        }

        // Find the closest valid tile
        let closest_tile_entity = valid_tiles
            .into_iter()
            .min_by(|(_, _, distance_a), (_, _, distance_b)| {
                distance_a.partial_cmp(distance_b).unwrap()
            })
            .map(|(entity, _, _)| entity);

        let Some(tile_entity) = closest_tile_entity else {
            continue;
        };
        let Ok((_, tile_global_transform)) = q_tiles.get(tile_entity)
        else {
            continue;
        };
        let tile_pos = tile_global_transform.translation();

        let Some(item_registry_asset) = item_registry.get() else {
            continue;
        };
        let Some(item_meta) =
            item_registry_asset.get(&selected_tower)
        else {
            continue;
        };
        if item_meta.item_type != ItemType::Tower {
            continue;
        }
        if !inventory.remove_tower(&selected_tower, 1) {
            continue;
        }

        // Mark tile as occupied
        commands.entity(tile_entity).insert(TileOccupied);

        // Spawn turret
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(0.3, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: BLUE.into(),
                ..default()
            })),
            Transform::from_translation(tile_pos + Vec3::Y * 0.5),
            RigidBody::Static,
            Collider::cylinder(0.3, 1.0),
            Item {
                id: selected_tower.clone(),
                quantity: 1,
            },
            PlacedBy(player_entity),
        ));

        commands.entity(player_entity).remove::<InPlacementMode>();
        for preview in q_previews.iter() {
            commands.entity(preview).despawn();
        }
    }
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct PlacementTile;

#[derive(Component, Clone, Debug)]
pub struct TileOccupied;

#[derive(Component)]
pub struct InPlacementMode;

#[derive(Component)]
pub struct Preview;

#[derive(Component, Deref, Default, Debug)]
#[relationship_target(relationship = PlacedBy)]
pub struct PlacedTurrets(Vec<Entity>);

#[derive(Component, Deref, Debug)]
#[relationship(relationship_target = PlacedTurrets)]
pub struct PlacedBy(Entity);
