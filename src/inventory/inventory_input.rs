use crate::action::PlayerAction;
use crate::action::TargetAction;
use crate::interaction::InteractionPlayer;
use crate::inventory::Inventory;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub(super) struct InventoryInputPlugin;

impl Plugin for InventoryInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cycle_selected_item);
    }
}

/// Cycle through selected items in the inventory for players
fn cycle_selected_item(
    mut q_players: Query<
        (&mut Inventory, &TargetAction),
        With<InteractionPlayer>,
    >,
    q_actions: Query<&ActionState<PlayerAction>>,
) {
    for (mut inventory, target_action) in q_players.iter_mut() {
        let Ok(action_state) = q_actions.get(target_action.get())
        else {
            continue;
        };

        cycle_tower_selection_for_player(
            action_state,
            &mut inventory,
        );
    }
}

fn cycle_tower_selection_for_player(
    action_state: &ActionState<PlayerAction>,
    inventory: &mut Inventory,
) {
    // Get available towers
    let mut available_towers: Vec<String> = inventory
        .towers
        .iter()
        .filter(|(_, count)| **count > 0)
        .map(|(id, _)| id.clone())
        .collect();
    available_towers.sort();

    // No towers available will clear selection
    if available_towers.is_empty() {
        inventory.selected_tower = None;
        return;
    }

    // Always ensure a valid selection
    let current_valid = inventory
        .selected_tower
        .as_ref()
        .map(|tower| available_towers.contains(tower))
        .unwrap_or(false);

    if !current_valid {
        // No valid selection, pick first available
        inventory.selected_tower = Some(available_towers[0].clone());
        return;
    }

    // Only process cycling if there are multiple towers
    if available_towers.len() > 1 {
        if action_state.just_pressed(&PlayerAction::CycleNext) {
            cycle_to_next_tower(
                &mut inventory.selected_tower,
                &available_towers,
            );
        } else if action_state.just_pressed(&PlayerAction::CyclePrev)
        {
            cycle_to_prev_tower(
                &mut inventory.selected_tower,
                &available_towers,
            );
        }
    }
}

fn cycle_to_next_tower(
    selected_tower: &mut Option<String>,
    available_towers: &[String],
) {
    if let Some(current) = selected_tower {
        if let Some(current_index) =
            available_towers.iter().position(|t| t == current)
        {
            let next_index =
                (current_index + 1) % available_towers.len();
            *selected_tower =
                Some(available_towers[next_index].clone());
        }
    }
}

fn cycle_to_prev_tower(
    selected_tower: &mut Option<String>,
    available_towers: &[String],
) {
    if let Some(current) = selected_tower {
        if let Some(current_index) =
            available_towers.iter().position(|t| t == current)
        {
            let prev_index = if current_index == 0 {
                available_towers.len() - 1
            } else {
                current_index - 1
            };
            *selected_tower =
                Some(available_towers[prev_index].clone());
        }
    }
}
