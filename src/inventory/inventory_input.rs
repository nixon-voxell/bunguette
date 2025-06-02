use crate::action::PlayerAction;
use crate::action::TargetAction;
use crate::interaction::InteractionPlayer;
use crate::inventory::Inventory;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

// TODO: Test after implement tower selection
pub(super) struct InventoryInputPlugin;

impl Plugin for InventoryInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, cycle_selected_item);
    }
}

/// System to cycle through selected items in the inventory for players with interaction capability.
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
    // Get available tower IDs (sorted for consistent ordering)
    let mut available_towers: Vec<String> =
        inventory.towers.keys().cloned().collect();
    available_towers.sort();

    if available_towers.is_empty() {
        inventory.selected_tower = None;
        return;
    }

    // Handle CycleNext
    if action_state.just_pressed(&PlayerAction::CycleNext) {
        let new_selection = if let Some(ref current_tower) =
            inventory.selected_tower
        {
            if let Some(current_index) = available_towers
                .iter()
                .position(|t| t == current_tower)
            {
                let next_index =
                    (current_index + 1) % available_towers.len();
                available_towers[next_index].clone()
            } else {
                available_towers[0].clone()
            }
        } else {
            available_towers[0].clone()
        };

        inventory.selected_tower = Some(new_selection.clone());
        info!("Selected next tower: {}", new_selection);
    }

    // Handle CyclePrev
    if action_state.just_pressed(&PlayerAction::CyclePrev) {
        let new_selection = if let Some(ref current_tower) =
            inventory.selected_tower
        {
            if let Some(current_index) = available_towers
                .iter()
                .position(|t| t == current_tower)
            {
                let prev_index = if current_index == 0 {
                    available_towers.len() - 1
                } else {
                    current_index - 1
                };
                available_towers[prev_index].clone()
            } else {
                available_towers[0].clone()
            }
        } else {
            available_towers[0].clone()
        };

        inventory.selected_tower = Some(new_selection.clone());
        info!("Selected previous tower: {}", new_selection);
    }

    // Initialize selection if none exists
    if inventory.selected_tower.is_none()
        && !available_towers.is_empty()
    {
        inventory.selected_tower = Some(available_towers[0].clone());
        info!("Initialized tower selection: {}", available_towers[0]);
    }
}
