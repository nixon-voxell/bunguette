use super::{
    Consumable, ConsumeEvent, DropEvent, Inventory, Item,
    ItemRegistry, PickupEvent, Pickupable,
};
use crate::interaction::{InteractionPlayer, MarkedItem};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

// TODO: Refactor to use PlayerAction
pub(super) struct InventoryInputPlugin;

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
                move_item_input,
            ),
        )
        .init_resource::<MoveItemState>();
    }
}

#[derive(Resource, Default)]
struct MoveItemState {
    pending_move: Option<(Entity, usize)>, // (item_entity, source_index)
}

fn cycle_selected_item(
    keys: Res<ButtonInput<KeyCode>>,
    mut scroll_events: EventReader<MouseWheel>,
    mut q_inventories: Query<&mut Inventory, With<InteractionPlayer>>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    q_items: Query<&Item>,
) {
    // Check if Alt is held down
    // TODO: Use PlayerAction instead of KeyCode (Implement InventoryModifier)
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
            // If inventory is empty, set selected_index to None and return
            if inventory.items.is_empty() {
                inventory.selected_index = None;
                return;
            }

            // Get indices of valid items (non-PLACEHOLDER with Item component)
            let valid_indices: Vec<usize> = inventory
                .items
                .iter()
                .enumerate()
                .filter(|&(_, &item_entity)| {
                    item_entity != Entity::PLACEHOLDER
                        && q_items.get(item_entity).is_ok()
                })
                .map(|(i, _)| i)
                .collect();

            if valid_indices.is_empty() {
                inventory.selected_index = None;
                return;
            }

            // Handle scroll events
            for event in scroll_events.read() {
                let scroll_delta = match event.unit {
                    MouseScrollUnit::Line => event.y,
                    MouseScrollUnit::Pixel => event.y / 100.0,
                };

                let current = inventory.selected_index.unwrap_or(0);
                let current_valid_pos = valid_indices
                    .iter()
                    .position(|&i| i == current)
                    .unwrap_or(0);

                let new_valid_pos = if scroll_delta > 0.0 {
                    // Scroll up - go to previous valid item
                    if current_valid_pos == 0 {
                        valid_indices.len() - 1
                    } else {
                        current_valid_pos - 1
                    }
                } else if scroll_delta < 0.0 {
                    // Scroll down - go to next valid item
                    if current_valid_pos >= valid_indices.len() - 1 {
                        0
                    } else {
                        current_valid_pos + 1
                    }
                } else {
                    current_valid_pos // No change if scroll delta is 0
                };

                let new_index = valid_indices[new_valid_pos];
                if new_index != current
                    || inventory.selected_index.is_none()
                {
                    inventory.selected_index = Some(new_index);
                    info!(
                        "Selected item slot {} for player {:?}",
                        new_index + 1,
                        player_entity
                    );
                    break; // Only process one scroll event per frame
                }
            }

            // Ensure selected_index is valid if not set
            if inventory.selected_index.is_none()
                && !valid_indices.is_empty()
            {
                inventory.selected_index = Some(valid_indices[0]);
                info!(
                    "Initialized selected item slot {} for player {:?}",
                    valid_indices[0] + 1,
                    player_entity
                );
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

/// Handles moving items between slots (M key to initiate move, Escape to cancel)
// TODO: Refactor to use PlayerAction
fn move_item_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut move_state: ResMut<MoveItemState>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    mut q_inventories: Query<&mut Inventory, With<InteractionPlayer>>,
    item_registry: Res<ItemRegistry>,
    q_items: Query<&Item>,
) {
    let player_entity = q_players.iter().next();
    if player_entity.is_none() {
        return;
    }
    let player_entity = player_entity.unwrap();

    if let Ok(mut inventory) = q_inventories.get_mut(player_entity) {
        // Initiate or complete move with 'M' key
        if keys.just_pressed(KeyCode::KeyM) {
            if move_state.pending_move.is_none() {
                // Start move
                if let Some(slot_index) = inventory.selected_index {
                    if let Some(&item_entity) =
                        inventory.items.get(slot_index)
                    {
                        if item_entity != Entity::PLACEHOLDER {
                            move_state.pending_move =
                                Some((item_entity, slot_index));
                            let item_name = q_items
                                .get(item_entity)
                                .ok()
                                .and_then(|item| {
                                    item_registry
                                        .by_id
                                        .get(&item.id)
                                        .map(|meta| {
                                            meta.name.as_str()
                                        })
                                })
                                .unwrap_or("Unknown");
                            info!(
                                "Initiated move for item {} in slot {} for player {:?}",
                                item_name,
                                slot_index + 1,
                                player_entity
                            );
                        } else {
                            info!(
                                "No item in selected slot {} to move for player {:?}",
                                slot_index + 1,
                                player_entity
                            );
                        }
                    } else {
                        info!(
                            "No item selected to move for player {:?}",
                            player_entity
                        );
                    }
                } else {
                    info!(
                        "No slot selected to initiate move for player {:?}",
                        player_entity
                    );
                }
            } else {
                // Complete move
                if let Some((item_entity, source_index)) =
                    move_state.pending_move
                {
                    if let Some(target_index) =
                        inventory.selected_index
                    {
                        if source_index != target_index
                            && target_index < inventory.capacity
                        {
                            // Ensure inventory has enough slots allocated
                            while inventory.items.len()
                                <= target_index
                            {
                                inventory
                                    .items
                                    .push(Entity::PLACEHOLDER);
                            }
                            // Swap or move
                            let target_item =
                                inventory.items[target_index];
                            inventory.items[source_index] =
                                target_item;
                            inventory.items[target_index] =
                                item_entity;
                            inventory.selected_index =
                                Some(target_index);
                            let item_name = q_items
                                .get(item_entity)
                                .ok()
                                .and_then(|item| {
                                    item_registry
                                        .by_id
                                        .get(&item.id)
                                        .map(|meta| {
                                            meta.name.as_str()
                                        })
                                })
                                .unwrap_or("Unknown");
                            if target_item == Entity::PLACEHOLDER {
                                info!(
                                    "Moved item {} from slot {} to empty slot {} for player {:?}",
                                    item_name,
                                    source_index + 1,
                                    target_index + 1,
                                    player_entity
                                );
                            } else {
                                let target_item_name = q_items
                                    .get(target_item)
                                    .ok()
                                    .and_then(|item| {
                                        item_registry
                                            .by_id
                                            .get(&item.id)
                                            .map(|meta| {
                                                meta.name.as_str()
                                            })
                                    })
                                    .unwrap_or("Unknown");
                                info!(
                                    "Swapped item {} in slot {} with item {} in slot {} for player {:?}",
                                    item_name,
                                    source_index + 1,
                                    target_item_name,
                                    target_index + 1,
                                    player_entity
                                );
                            }
                            move_state.pending_move = None;
                        } else if source_index == target_index {
                            info!(
                                "Move canceled: same slot {} selected for player {:?}",
                                source_index + 1,
                                player_entity
                            );
                            move_state.pending_move = None;
                        } else {
                            warn!(
                                "Target slot {} exceeds inventory capacity {} for player {:?}",
                                target_index + 1,
                                inventory.capacity,
                                player_entity
                            );
                        }
                    } else {
                        warn!(
                            "No target slot selected to complete move for player {:?}",
                            player_entity
                        );
                    }
                }
            }
        }

        // Cancel move with 'Escape' key
        if keys.just_pressed(KeyCode::Escape)
            && move_state.pending_move.is_some()
        {
            move_state.pending_move = None;
            info!(
                "Cancelled item move for player {:?}",
                player_entity
            );
        }
    }
}

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
                        if item_entity == Entity::PLACEHOLDER {
                            info!("  {}: (empty)", i + 1);
                        } else if let Ok(item) =
                            q_items.get(item_entity)
                        {
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
