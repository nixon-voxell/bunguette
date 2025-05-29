use super::{
    Consumable, ConsumeEvent, DropEvent, Inventory, Item,
    ItemRegistry, PickupEvent, Pickupable,
};
use crate::action::PlayerAction;
use crate::interaction::{InteractionPlayer, MarkedItem};
use crate::player::{QueryPlayerA, QueryPlayerB};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub(super) struct InventoryInputPlugin;

impl Plugin for InventoryInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                pickup_input,
                drop_input,
                consume_input,
                cycle_selected_item,
                move_item_input,
            ),
        )
        .init_resource::<MoveItemStateA>()
        .init_resource::<MoveItemStateB>();
    }
}

#[derive(Resource, Default)]
struct MoveItemStateA {
    pending_move: Option<(Entity, usize)>,
}

#[derive(Resource, Default)]
struct MoveItemStateB {
    pending_move: Option<(Entity, usize)>,
}

fn cycle_selected_item(
    mut q_player_a: QueryPlayerA<
        (&ActionState<PlayerAction>, &mut Inventory),
        With<InteractionPlayer>,
    >,
    mut q_player_b: QueryPlayerB<
        (&ActionState<PlayerAction>, &mut Inventory),
        With<InteractionPlayer>,
    >,
    q_items: Query<&Item>,
) {
    // Handle Player A
    if let Ok((action_state, mut inventory)) = q_player_a.single_mut()
    {
        cycle_inventory_for_player(
            action_state,
            &mut inventory,
            &q_items,
            "A",
        );
    }

    // Handle Player B
    if let Ok((action_state, mut inventory)) = q_player_b.single_mut()
    {
        cycle_inventory_for_player(
            action_state,
            &mut inventory,
            &q_items,
            "B",
        );
    }
}

fn cycle_inventory_for_player(
    action_state: &ActionState<PlayerAction>,
    inventory: &mut Inventory,
    q_items: &Query<&Item>,
    player_name: &str,
) {
    // Check if InventoryModifier is held down
    if !action_state.pressed(&PlayerAction::InventoryModifier) {
        return;
    }

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

    let mut changed = false;

    // Handle CycleNext
    if action_state.just_pressed(&PlayerAction::CycleNext) {
        let current = inventory.selected_index.unwrap_or(0);
        let current_valid_pos = valid_indices
            .iter()
            .position(|&i| i == current)
            .unwrap_or(0);

        let new_valid_pos =
            if current_valid_pos >= valid_indices.len() - 1 {
                0
            } else {
                current_valid_pos + 1
            };

        let new_index = valid_indices[new_valid_pos];
        if new_index != current || inventory.selected_index.is_none()
        {
            inventory.selected_index = Some(new_index);
            changed = true;
        }
    }

    // Handle CyclePrev
    if action_state.just_pressed(&PlayerAction::CyclePrev) {
        let current = inventory.selected_index.unwrap_or(0);
        let current_valid_pos = valid_indices
            .iter()
            .position(|&i| i == current)
            .unwrap_or(0);

        let new_valid_pos = if current_valid_pos == 0 {
            valid_indices.len() - 1
        } else {
            current_valid_pos - 1
        };

        let new_index = valid_indices[new_valid_pos];
        if new_index != current || inventory.selected_index.is_none()
        {
            inventory.selected_index = Some(new_index);
            changed = true;
        }
    }

    if changed {
        info!(
            "Player {} selected item slot {}",
            player_name,
            inventory.selected_index.unwrap() + 1
        );
    }

    // Ensure selected_index is valid if not set
    if inventory.selected_index.is_none() && !valid_indices.is_empty()
    {
        inventory.selected_index = Some(valid_indices[0]);
        info!(
            "Player {} initialized selected item slot {}",
            player_name,
            valid_indices[0] + 1
        );
    }
}

/// Handles pickup input for both players
fn pickup_input(
    mut commands: Commands,
    q_player_a: QueryPlayerA<
        (
            Entity,
            &ActionState<PlayerAction>,
            &MarkedItem,
            Option<&Inventory>,
        ),
        With<InteractionPlayer>,
    >,
    q_player_b: QueryPlayerB<
        (
            Entity,
            &ActionState<PlayerAction>,
            &MarkedItem,
            Option<&Inventory>,
        ),
        With<InteractionPlayer>,
    >,
    q_pickupable: Query<&Pickupable>,
) {
    // Handle Player A
    if let Ok((player_entity, action_state, marked, inventory)) =
        q_player_a.single()
    {
        if action_state.just_pressed(&PlayerAction::Pickup) {
            if inventory.is_none() {
                commands.entity(player_entity).insert(Inventory {
                    capacity: 9,
                    ..Default::default()
                });
            }
            handle_pickup_for_player(
                &mut commands,
                player_entity,
                marked,
                &q_pickupable,
            );
        }
    }

    // Handle Player B
    if let Ok((player_entity, action_state, marked, inventory)) =
        q_player_b.single()
    {
        if action_state.just_pressed(&PlayerAction::Pickup) {
            if inventory.is_none() {
                commands.entity(player_entity).insert(Inventory {
                    capacity: 9,
                    ..Default::default()
                });
            }
            handle_pickup_for_player(
                &mut commands,
                player_entity,
                marked,
                &q_pickupable,
            );
        }
    }
}

fn handle_pickup_for_player(
    commands: &mut Commands,
    player_entity: Entity,
    marked: &MarkedItem,
    q_pickupable: &Query<&Pickupable>,
) {
    info!(
        "handle_pickup_for_player called for player {:?}",
        player_entity
    );

    if let Some(target) = marked.0 {
        info!("Marked item: {:?}", target);

        // Check if the marked item is pickupable
        if q_pickupable.get(target).is_ok() {
            info!(
                "Triggering PickupEvent for player {:?} and item {:?}",
                player_entity, target
            );
            commands.trigger_targets(
                PickupEvent { item: target },
                player_entity,
            );
        } else {
            warn!("Item {:?} is not pickupable", target);
        }
    } else {
        warn!(
            "No marked item to pick up for player {:?}",
            player_entity
        );
    }
}

/// Handles drop input for both players
fn drop_input(
    mut commands: Commands,
    q_player_a: QueryPlayerA<
        (Entity, &ActionState<PlayerAction>, &Inventory),
        With<InteractionPlayer>,
    >,
    q_player_b: QueryPlayerB<
        (Entity, &ActionState<PlayerAction>, &Inventory),
        With<InteractionPlayer>,
    >,
) {
    // Handle Player A
    if let Ok((player_entity, action_state, inventory)) =
        q_player_a.single()
    {
        if action_state.just_pressed(&PlayerAction::Drop) {
            handle_drop_for_player(
                &mut commands,
                player_entity,
                inventory,
                "A",
            );
        }
    }

    // Handle Player B
    if let Ok((player_entity, action_state, inventory)) =
        q_player_b.single()
    {
        if action_state.just_pressed(&PlayerAction::Drop) {
            handle_drop_for_player(
                &mut commands,
                player_entity,
                inventory,
                "B",
            );
        }
    }
}

fn handle_drop_for_player(
    commands: &mut Commands,
    player_entity: Entity,
    inventory: &Inventory,
    player_name: &str,
) {
    if let Some(slot_index) = inventory.selected_index {
        if let Some(&item_entity) = inventory.items.get(slot_index) {
            if item_entity != Entity::PLACEHOLDER {
                commands.trigger_targets(
                    DropEvent { item: item_entity },
                    player_entity,
                );
                info!(
                    "Player {} dropped item from selected slot {}",
                    player_name,
                    slot_index + 1
                );
                return;
            }
        }
    }

    // Fallback to first non-placeholder item
    for &item_entity in inventory.items.iter() {
        if item_entity != Entity::PLACEHOLDER {
            commands.trigger_targets(
                DropEvent { item: item_entity },
                player_entity,
            );
            info!(
                "Player {} dropped first item from inventory (no slot selected)",
                player_name
            );
            return;
        }
    }
}

/// Handles consume input for both players
fn consume_input(
    mut commands: Commands,
    q_player_a: QueryPlayerA<
        (Entity, &ActionState<PlayerAction>, &Inventory),
        With<InteractionPlayer>,
    >,
    q_player_b: QueryPlayerB<
        (Entity, &ActionState<PlayerAction>, &Inventory),
        With<InteractionPlayer>,
    >,
    q_consumable: Query<&Consumable>,
) {
    // Handle Player A
    if let Ok((player_entity, action_state, inventory)) =
        q_player_a.single()
    {
        if action_state.just_pressed(&PlayerAction::Consume) {
            handle_consume_for_player(
                &mut commands,
                player_entity,
                inventory,
                &q_consumable,
                "A",
            );
        }
    }

    // Handle Player B
    if let Ok((player_entity, action_state, inventory)) =
        q_player_b.single()
    {
        if action_state.just_pressed(&PlayerAction::Consume) {
            handle_consume_for_player(
                &mut commands,
                player_entity,
                inventory,
                &q_consumable,
                "B",
            );
        }
    }
}

fn handle_consume_for_player(
    commands: &mut Commands,
    player_entity: Entity,
    inventory: &Inventory,
    q_consumable: &Query<&Consumable>,
    player_name: &str,
) {
    // Try selected item first
    if let Some(slot_index) = inventory.selected_index {
        if let Some(&item_entity) = inventory.items.get(slot_index) {
            if item_entity != Entity::PLACEHOLDER {
                if q_consumable.get(item_entity).is_ok() {
                    commands.trigger_targets(
                        ConsumeEvent { item: item_entity },
                        player_entity,
                    );
                    info!(
                        "Player {} consumed item from selected slot {}",
                        player_name,
                        slot_index + 1
                    );
                    return;
                } else {
                    info!(
                        "Player {} selected item in slot {} is not consumable",
                        player_name,
                        slot_index + 1
                    );
                    return;
                }
            }
        }
    }

    // Fallback to first consumable item
    for &item_entity in inventory.items.iter() {
        if item_entity != Entity::PLACEHOLDER
            && q_consumable.get(item_entity).is_ok()
        {
            commands.trigger_targets(
                ConsumeEvent { item: item_entity },
                player_entity,
            );
            info!(
                "Player {} consumed first consumable item (no slot selected)",
                player_name
            );
            return;
        }
    }
}

/// Handles moving items between slots for both players
fn move_item_input(
    mut q_player_a: QueryPlayerA<
        (Entity, &ActionState<PlayerAction>, &mut Inventory),
        With<InteractionPlayer>,
    >,
    mut q_player_b: QueryPlayerB<
        (Entity, &ActionState<PlayerAction>, &mut Inventory),
        With<InteractionPlayer>,
    >,
    mut move_state_a: ResMut<MoveItemStateA>,
    mut move_state_b: ResMut<MoveItemStateB>,
    item_registry: Res<ItemRegistry>,
    q_items: Query<&Item>,
) {
    // Handle Player A
    if let Ok((player_entity, action_state, mut inventory)) =
        q_player_a.single_mut()
    {
        handle_move_item_for_player(
            player_entity,
            action_state,
            &mut inventory,
            &mut move_state_a.pending_move,
            &item_registry,
            &q_items,
            "A",
        );
    }

    // Handle Player B
    if let Ok((player_entity, action_state, mut inventory)) =
        q_player_b.single_mut()
    {
        handle_move_item_for_player(
            player_entity,
            action_state,
            &mut inventory,
            &mut move_state_b.pending_move,
            &item_registry,
            &q_items,
            "B",
        );
    }
}

fn handle_move_item_for_player(
    _player_entity: Entity,
    action_state: &ActionState<PlayerAction>,
    inventory: &mut Inventory,
    pending_move: &mut Option<(Entity, usize)>,
    item_registry: &ItemRegistry,
    q_items: &Query<&Item>,
    player_name: &str,
) {
    // Initiate or complete move with MoveItem action
    if action_state.just_pressed(&PlayerAction::MoveItem) {
        if pending_move.is_none() {
            // Start move
            if let Some(slot_index) = inventory.selected_index {
                if let Some(&item_entity) =
                    inventory.items.get(slot_index)
                {
                    if item_entity != Entity::PLACEHOLDER {
                        *pending_move =
                            Some((item_entity, slot_index));
                        let item_name = q_items
                            .get(item_entity)
                            .ok()
                            .and_then(|item| {
                                item_registry
                                    .by_id
                                    .get(&item.id)
                                    .map(|meta| meta.name.as_str())
                            })
                            .unwrap_or("Unknown");
                        info!(
                            "Player {} initiated move for item {} in slot {}",
                            player_name,
                            item_name,
                            slot_index + 1
                        );
                    } else {
                        info!(
                            "Player {} no item in selected slot {} to move",
                            player_name,
                            slot_index + 1
                        );
                    }
                } else {
                    info!(
                        "Player {} no item selected to move",
                        player_name
                    );
                }
            } else {
                info!(
                    "Player {} no slot selected to initiate move",
                    player_name
                );
            }
        } else {
            // Complete move
            if let Some((item_entity, source_index)) = *pending_move {
                if let Some(target_index) = inventory.selected_index {
                    if source_index != target_index
                        && target_index < inventory.capacity
                    {
                        // Ensure inventory has enough slots allocated
                        while inventory.items.len() <= target_index {
                            inventory.items.push(Entity::PLACEHOLDER);
                        }

                        // Swap or move
                        let target_item =
                            inventory.items[target_index];
                        inventory.items[source_index] = target_item;
                        inventory.items[target_index] = item_entity;
                        inventory.selected_index = Some(target_index);

                        let item_name = q_items
                            .get(item_entity)
                            .ok()
                            .and_then(|item| {
                                item_registry
                                    .by_id
                                    .get(&item.id)
                                    .map(|meta| meta.name.as_str())
                            })
                            .unwrap_or("Unknown");

                        if target_item == Entity::PLACEHOLDER {
                            info!(
                                "Player {} moved item {} from slot {} to empty slot {}",
                                player_name,
                                item_name,
                                source_index + 1,
                                target_index + 1
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
                                "Player {} swapped item {} in slot {} with item {} in slot {}",
                                player_name,
                                item_name,
                                source_index + 1,
                                target_item_name,
                                target_index + 1
                            );
                        }
                        *pending_move = None;
                    } else if source_index == target_index {
                        info!(
                            "Player {} move canceled: same slot {} selected",
                            player_name,
                            source_index + 1
                        );
                        *pending_move = None;
                    } else {
                        warn!(
                            "Player {} target slot {} exceeds inventory capacity {}",
                            player_name,
                            target_index + 1,
                            inventory.capacity
                        );
                    }
                } else {
                    warn!(
                        "Player {} no target slot selected to complete move",
                        player_name
                    );
                }
            }
        }
    }
    // TODO: Cancel move with CancelMoveItem action
}
