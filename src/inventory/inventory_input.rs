use super::{
    Consumable, ConsumeEvent, DropEvent, Inventory, PickupEvent,
    Pickupable,
};
use crate::interaction::{InteractionPlayer, MarkedItem};
use bevy::prelude::*;

pub struct InventoryInputPlugin;

impl Plugin for InventoryInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                pickup_input_system,
                drop_input_system,
                consume_input_system,
                debug_inventory_system,
            ),
        );
    }
}

/// Handles pickup input (E key when looking at a pickupable item)
fn pickup_input_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    q_players: Query<(Entity, &MarkedItem), With<InteractionPlayer>>,
    q_pickupable: Query<&Pickupable>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        for (player_entity, marked) in q_players.iter() {
            if let Some(target) = marked.0 {
                // Check if the marked item is pickupable
                if q_pickupable.get(target).is_ok() {
                    commands.trigger_targets(
                        PickupEvent { item: target },
                        player_entity,
                    );
                }
            }
        }
    }
}

/// Handles drop input (Q key to drop first item in inventory)
fn drop_input_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    q_inventories: Query<&Inventory>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        for player_entity in q_players.iter() {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                if let Some(&first_item) = inventory.0.first() {
                    commands.trigger_targets(
                        DropEvent { item: first_item },
                        player_entity,
                    );
                }
            }
        }
    }
}

/// Handles consume input (C key to consume first consumable item)
fn consume_input_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    q_inventories: Query<&Inventory>,
    q_consumable: Query<&Consumable>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        for player_entity in q_players.iter() {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                // Find first consumable item
                for &item_entity in inventory.0.iter() {
                    if q_consumable.get(item_entity).is_ok() {
                        commands.trigger_targets(
                            ConsumeEvent { item: item_entity },
                            player_entity,
                        );
                        break; // Only consume one item at a time
                    }
                }
            }
        }
    }
}

/// Debug system to show inventory contents (I key)
fn debug_inventory_system(
    keys: Res<ButtonInput<KeyCode>>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    q_inventories: Query<&Inventory>,
    q_items: Query<&crate::inventory::Item>,
) {
    if keys.just_pressed(KeyCode::KeyI) {
        for player_entity in q_players.iter() {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                info!("=== Player {:?} Inventory ===", player_entity);
                if inventory.0.is_empty() {
                    info!("  (empty)");
                } else {
                    for (i, &item_entity) in
                        inventory.0.iter().enumerate()
                    {
                        if let Ok(item) = q_items.get(item_entity) {
                            info!(
                                "  {}: {}x {} (id: {})",
                                i + 1,
                                item.quantity,
                                item.name,
                                item.id
                            );
                        } else {
                            info!(
                                "  {}: Invalid item {:?}",
                                i + 1,
                                item_entity
                            );
                        }
                    }
                }
                info!("=========================");
            } else {
                info!("Player {:?} has no inventory", player_entity);
            }
        }
    }
}
