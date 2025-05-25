use avian3d::prelude::*;
use bevy::color::palettes::tailwind::SKY_300;
use bevy::prelude::*;
use bevy_mod_outline::{
    InheritOutline, OutlineMode, OutlineStencil, OutlineVolume,
};

mod grab;

use crate::physics::GameLayer;

const MARK_COLOR: Color = Color::Srgba(SKY_300);
// const GRABBED_COLOR: Color = Color::Srgba(EMERALD_500);

pub(super) struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bevy_mod_outline::OutlinePlugin,
            grab::GrabPlugin,
        ));

        app.add_systems(
            Update,
            (
                setup_interactable_outline,
                detect_interactables,
                mark_item,
            ),
        );

        app.register_type::<Interactable>()
            .register_type::<InteractionPlayer>();
    }
}

fn detect_interactables(
    mut q_players: Query<
        (&InteractionPlayer, &mut MarkedItem, Entity),
        (Changed<GlobalTransform>, Without<Occupied>),
    >,
    q_global_transforms: Query<&GlobalTransform>,
    spatial_query: SpatialQuery,
) -> Result {
    for (player, mut marked_item, entity) in q_players.iter_mut() {
        let player_transform =
            q_global_transforms.get(entity).map_err(|_|
                "`InteractionPlayer` should have a global transform!",
            )?;

        let player_translation = player_transform.translation();

        let item_entities = spatial_query.shape_intersections(
            &Collider::sphere(player.range),
            player_translation,
            Quat::IDENTITY,
            &SpatialQueryFilter::from_mask(GameLayer::Interactable),
        );

        // No items around.
        if item_entities.is_empty() {
            marked_item.0 = None;
            continue;
        }

        // Find the closest items and keep track of items within the boundary range.
        let mut closest_idx = 0;
        let mut closest_dist = f32::MAX;

        let mut boundary_entities = Vec::new();

        for (i, &item_entity) in item_entities.iter().enumerate() {
            // Ignore self.
            if item_entity == entity {
                continue;
            }

            let Ok(item_translation) = q_global_transforms
                .get(item_entity)
                .map(|g| g.translation())
            else {
                continue;
            };

            let dist =
                item_translation.distance_squared(player_translation);

            if dist < closest_dist {
                closest_idx = i;
                closest_dist = dist;
            }

            if dist < player.boundary_range {
                boundary_entities.push((i, item_translation));
            }
        }

        // Find the one that is closest to the front of the player
        // for boundary items.
        if boundary_entities.is_empty() == false {
            let player_forward = player_transform.forward().as_vec3();

            let mut closest_angle = -1.0;
            closest_idx = 0;

            for (i, item_translation) in boundary_entities {
                let angle = (item_translation - player_translation)
                    .normalize()
                    .dot(player_forward);

                if angle > closest_angle {
                    closest_idx = i;
                    closest_angle = angle;
                }
            }
        }

        marked_item.0 = Some(item_entities[closest_idx]);
    }

    Ok(())
}

fn mark_item(
    mut q_marked_items: Query<
        (&MarkedItem, &mut PrevMarkedItem),
        Changed<MarkedItem>,
    >,
    mut q_outlines: Query<&mut OutlineVolume>,
) {
    for (marked, mut prev_marked) in q_marked_items.iter_mut() {
        if let Some(mut outline) =
            prev_marked.0.and_then(|e| q_outlines.get_mut(e).ok())
        {
            outline.visible = false;
        }

        if let Some(mut outline) =
            marked.0.and_then(|e| q_outlines.get_mut(e).ok())
        {
            outline.visible = true;
            outline.colour = MARK_COLOR;
        }

        prev_marked.0 = marked.0;
    }
}

fn setup_interactable_outline(
    q_interactables: Query<Entity, Added<Interactable>>,
    mut commands: Commands,
    q_meshes: Query<(), With<Mesh3d>>,
    q_children: Query<&Children>,
) {
    for entity in q_interactables.iter() {
        const VOLUME: OutlineVolume = OutlineVolume {
            width: 2.0,
            visible: false,
            colour: MARK_COLOR,
        };

        commands.entity(entity).insert(CollisionLayers::new(
            GameLayer::Interactable,
            LayerMask::ALL,
        ));

        if q_meshes.contains(entity) {
            commands
                .entity(entity)
                .insert((VOLUME, OutlineMode::FloodFlat));
        } else {
            commands.entity(entity).insert((
                VOLUME,
                OutlineMode::FloodFlat,
                OutlineStencil::default(),
            ));

            for child in q_children.iter_descendants(entity) {
                commands.entity(child).insert(InheritOutline);
            }
        }
    }
}

/// An entity that can be interacted.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Interactable;

/// Previously marked item, mostly for ressetting the outline value.
#[derive(Component, Deref, DerefMut, Default, Debug, Clone, Copy)]
pub struct PrevMarkedItem(pub Option<Entity>);

#[derive(Component, Deref, DerefMut, Default, Debug, Clone, Copy)]
#[require(PrevMarkedItem)]
pub struct MarkedItem(pub Option<Entity>);

/// Entity that can perform interaction.
/// Raycast will happen from this player.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(MarkedItem)]
pub struct InteractionPlayer {
    /// The interaction radius.
    pub range: f32,
    /// The interaction boundary, anything that is
    /// closer than this range will be considered and ranked
    /// based on their direction.
    pub boundary_range: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct TestBundle {
    pub transform: Transform,
    pub visibility: Visibility,
}

/// Tags the player as occupied when holding an item.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Occupied;
