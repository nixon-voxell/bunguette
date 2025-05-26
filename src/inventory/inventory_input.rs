use super::{
    Consumable, ConsumeEvent, DropEvent, Inventory, Item,
    ItemRegistry, PickupEvent, Pickupable,
};
use crate::interaction::{InteractionPlayer, MarkedItem};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

pub struct InventoryInputPlugin;

impl Plugin for InventoryInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                pickup_input,
                drop_input,
                consume_input,
                debug_inventory_system,
                cycle_selected_item,
            ),
        );
    }
}

fn cycle_selected_item(
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    mut q_inventories: Query<&mut Inventory, With<InteractionPlayer>>,
    q_players: Query<Entity, With<InteractionPlayer>>,
) {
    // Check if Alt is held down
    let alt_held = keys.pressed(KeyCode::AltLeft)
        || keys.pressed(KeyCode::AltRight);

    if !alt_held {
        return;
    }

    let player_entity = q_players.iter().next();

    if let Some(player_entity) = player_entity {
        if let Ok(mut inventory) =
            q_inventories.get_mut(player_entity)
        {
            if inventory.items.is_empty() {
                inventory.selected_index = None;
                return;
            }

            let current = inventory.selected_index.unwrap_or(0);
            let max_index = inventory.items.len().saturating_sub(1);

            // Handle scroll events
            for event in scroll_events.read() {
                let scroll_delta = match event.unit {
                    MouseScrollUnit::Line => event.y,
                    MouseScrollUnit::Pixel => event.y / 100.0, // Convert pixels to reasonable line units
                };

                let new_index = if scroll_delta > 0.0 {
                    // Scroll up - go to previous item
                    if current == 0 { max_index } else { current - 1 }
                } else if scroll_delta < 0.0 {
                    // Scroll down - go to next item
                    if current >= max_index { 0 } else { current + 1 }
                } else {
                    current // No change if scroll delta is 0
                };

                if new_index != current {
                    inventory.selected_index = Some(new_index);
                    info!(
                        "Selected item slot {} for player {:?}",
                        new_index + 1,
                        player_entity
                    );
                    break; // Only process one scroll event per frame
                }
            }
        }
    }
}

/// Handles pickup input (E key when looking at a pickupable item)
fn pickup_input(
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

/// Handles drop input (Q key to drop selected item, or first item if none selected)
fn drop_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    q_inventories: Query<&Inventory>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        for player_entity in q_players.iter() {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                if let Some(slot_index) = inventory.selected_index {
                    if let Some(&item_entity) =
                        inventory.items.get(slot_index)
                    {
                        commands.trigger_targets(
                            DropEvent { item: item_entity },
                            player_entity,
                        );
                        info!(
                            "Dropped item from selected slot {}",
                            slot_index + 1
                        );
                        return;
                    }
                }
                if let Some(&first_item) = inventory.items.first() {
                    commands.trigger_targets(
                        DropEvent { item: first_item },
                        player_entity,
                    );
                    info!(
                        "Dropped first item from inventory (no slot selected)"
                    );
                    break;
                }
            }
        }
    }
}

/// Handles consume input (C key to consume selected item, or first consumable if none selected)
fn consume_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    q_inventories: Query<&Inventory>,
    q_consumable: Query<&Consumable>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        for player_entity in q_players.iter() {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                if let Some(slot_index) = inventory.selected_index {
                    if let Some(&item_entity) =
                        inventory.items.get(slot_index)
                    {
                        if q_consumable.get(item_entity).is_ok() {
                            commands.trigger_targets(
                                ConsumeEvent { item: item_entity },
                                player_entity,
                            );
                            info!(
                                "Consumed item from selected slot {}",
                                slot_index + 1
                            );
                            return;
                        } else {
                            info!(
                                "Selected item in slot {} is not consumable",
                                slot_index + 1
                            );
                            return;
                        }
                    }
                }
                for &item_entity in inventory.items.iter() {
                    if q_consumable.get(item_entity).is_ok() {
                        commands.trigger_targets(
                            ConsumeEvent { item: item_entity },
                            player_entity,
                        );
                        info!(
                            "Consumed first consumable item (no slot selected)"
                        );
                        return;
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
    q_items: Query<&Item>,
    item_registry: Res<ItemRegistry>,
) {
    if keys.just_pressed(KeyCode::KeyI) {
        for player_entity in q_players.iter() {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                info!("=== Player {:?} Inventory ===", player_entity);
                info!("  Capacity: {}", inventory.capacity);
                info!(
                    "  Selected slot: {:?}",
                    inventory.selected_index.map(|i| i + 1)
                );

                if inventory.items.is_empty() {
                    info!("  (empty)");
                } else {
                    for (i, &item_entity) in
                        inventory.items.iter().enumerate()
                    {
                        if let Ok(item) = q_items.get(item_entity) {
                            let item_name = item_registry
                                .by_id
                                .get(&item.id)
                                .map(|meta| meta.name.as_str())
                                .unwrap_or("Unknown Item");

                            let selected_marker = if inventory
                                .selected_index
                                == Some(i)
                            {
                                " [SELECTED]"
                            } else {
                                ""
                            };

                            info!(
                                "  {}: {}x {} (id: {}){}",
                                i + 1,
                                item.quantity,
                                item_name,
                                item.id,
                                selected_marker
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
