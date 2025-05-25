use super::{
    Consumable, ConsumeEvent, DropEvent, Inventory, Item,
    ItemRegistry, PickupEvent, Pickupable,
};
use crate::interaction::{InteractionPlayer, MarkedItem};
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
                slot_selection_input,
            ),
        )
        .init_resource::<SelectedSlot>();
    }
}

/// Handles slot selection input (number keys 1-9)
fn slot_selection_input(
    mut selected_slot: ResMut<SelectedSlot>,
    keys: Res<ButtonInput<KeyCode>>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    q_inventories: Query<&Inventory>,
) {
    // Map number keys to slot indices (1-9 maps to slots 0-8)
    let slot_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];

    // Check for number key presses
    for (i, &key) in slot_keys.iter().enumerate() {
        if keys.just_pressed(key) {
            // Find the appropriate player (prefer local player or use first player)
            let player_entity = q_players.iter().next();

            if let Some(player_entity) = player_entity {
                if let Ok(inventory) =
                    q_inventories.get(player_entity)
                {
                    // Only select slot if it's within inventory bounds and has an item
                    if i < inventory.0.len() {
                        selected_slot.slot_index = Some(i);
                        selected_slot.player_entity =
                            Some(player_entity);
                        info!(
                            "Selected slot {} for player {:?}",
                            i + 1,
                            player_entity
                        );
                    } else {
                        // Clear selection if slot is empty or out of bounds
                        selected_slot.slot_index = None;
                        selected_slot.player_entity = None;
                        info!(
                            "Deselected slot (slot {} is empty or invalid)",
                            i + 1
                        );
                    }
                }
            }
            break;
        }
    }

    // Clear selection with Escape key
    if keys.just_pressed(KeyCode::Escape) {
        selected_slot.slot_index = None;
        selected_slot.player_entity = None;
        info!("Cleared slot selection");
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
    selected_slot: Res<SelectedSlot>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    q_inventories: Query<&Inventory>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        // If we have a selected slot, use that player and slot
        if let (Some(slot_index), Some(player_entity)) =
            (selected_slot.slot_index, selected_slot.player_entity)
        {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                if let Some(&item_entity) =
                    inventory.0.get(slot_index)
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
        }

        // Fallback: drop first item from any player's inventory
        for player_entity in q_players.iter() {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                if let Some(&first_item) = inventory.0.first() {
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
    selected_slot: Res<SelectedSlot>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    q_inventories: Query<&Inventory>,
    q_consumable: Query<&Consumable>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        // If we have a selected slot, try to consume that specific item
        if let (Some(slot_index), Some(player_entity)) =
            (selected_slot.slot_index, selected_slot.player_entity)
        {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                if let Some(&item_entity) =
                    inventory.0.get(slot_index)
                {
                    // Check if the selected item is consumable
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
        }

        // Fallback: consume first consumable item from any player's inventory
        for player_entity in q_players.iter() {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                // Find first consumable item
                for &item_entity in inventory.0.iter() {
                    if q_consumable.get(item_entity).is_ok() {
                        commands.trigger_targets(
                            ConsumeEvent { item: item_entity },
                            player_entity,
                        );
                        info!(
                            "Consumed first consumable item (no slot selected)"
                        );
                        return; // Only consume one item at a time
                    }
                }
            }
        }
    }
}

/// Debug system to show inventory contents (I key)
fn debug_inventory_system(
    keys: Res<ButtonInput<KeyCode>>,
    selected_slot: Res<SelectedSlot>,
    q_players: Query<Entity, With<InteractionPlayer>>,
    q_inventories: Query<&Inventory>,
    q_items: Query<&Item>,
    item_registry: Res<ItemRegistry>,
) {
    if keys.just_pressed(KeyCode::KeyI) {
        for player_entity in q_players.iter() {
            if let Ok(inventory) = q_inventories.get(player_entity) {
                info!("=== Player {:?} Inventory ===", player_entity);

                // Show selected slot info
                if let (Some(slot_index), Some(selected_player)) = (
                    selected_slot.slot_index,
                    selected_slot.player_entity,
                ) {
                    if selected_player == player_entity {
                        info!("  Selected slot: {}", slot_index + 1);
                    }
                }

                if inventory.0.is_empty() {
                    info!("  (empty)");
                } else {
                    for (i, &item_entity) in
                        inventory.0.iter().enumerate()
                    {
                        if let Ok(item) = q_items.get(item_entity) {
                            let item_name = item_registry
                                .by_id
                                .get(&item.id)
                                .map(|meta| meta.name.as_str())
                                .unwrap_or("Unknown Item");

                            let selected_marker = if let (
                                Some(slot_index),
                                Some(selected_player),
                            ) = (
                                selected_slot.slot_index,
                                selected_slot.player_entity,
                            ) {
                                if selected_player == player_entity
                                    && slot_index == i
                                {
                                    " [SELECTED]"
                                } else {
                                    ""
                                }
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

/// Resource to track which inventory slot is currently selected
#[derive(Resource, Default)]
pub struct SelectedSlot {
    pub slot_index: Option<usize>,
    pub player_entity: Option<Entity>,
}
