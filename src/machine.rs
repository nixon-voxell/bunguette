use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use recipe::RecipeMeta;

use crate::action::{PlayerAction, TargetAction};
use crate::interaction::{InteractionPlayer, MarkedItem};
use crate::inventory::Inventory;
use crate::machine::recipe::RecipeRegistry;

mod machine_ui;
mod recipe;

pub(super) struct MachinePlugin;

impl Plugin for MachinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            machine_ui::MachineUiPlugin,
            recipe::RecipePlugin,
        ))
        .add_systems(Update, handle_player_machine_input)
        .add_systems(Update, update_cooking_machines)
        .add_observer(process_machine_interaction)
        .add_observer(handle_tower_production);
    }
}

/// Handle player interaction with machines
fn handle_player_machine_input(
    mut commands: Commands,
    q_players: Query<
        (Entity, &MarkedItem, &TargetAction),
        With<InteractionPlayer>,
    >,
    q_actions: Query<&ActionState<PlayerAction>>,
    q_machines: Query<&Machine>,
    q_inventories: Query<&Inventory>,
    recipe_registry: RecipeRegistry,
) {
    for (player_entity, marked_item, target_action) in
        q_players.iter()
    {
        let Ok(action_state) = q_actions.get(target_action.get())
        else {
            continue;
        };

        if !action_state.just_pressed(&PlayerAction::Interact) {
            continue;
        };

        let Some(machine_entity) = marked_item.0 else {
            continue;
        };

        let Ok(machine) = q_machines.get(machine_entity) else {
            continue;
        };

        if !matches!(machine.state, MachineState::Ready) {
            continue;
        }

        let Ok(inventory) = q_inventories.get(player_entity) else {
            info!("Player {:?} has no inventory", player_entity);
            continue;
        };

        if can_start_cooking(inventory, machine, &recipe_registry) {
            commands.trigger_targets(
                MachineInteractionEvent {
                    player: player_entity,
                },
                machine_entity,
            );
        } else {
            info!(
                "Player {:?} doesn't have required ingredients for recipe '{}'",
                player_entity, machine.recipe_id
            );
        }
    }
}

/// Check if player has all required ingredients
fn can_start_cooking(
    inventory: &Inventory,
    machine: &Machine,
    recipe_registry: &RecipeRegistry,
) -> bool {
    let Some(recipe) = machine.get_recipe(recipe_registry) else {
        warn!("Recipe '{}' not found in registry", machine.recipe_id);
        return false;
    };

    for ingredient in &recipe.ingredients {
        let available_quantity = inventory
            .get_all_ingredients()
            .get(&ingredient.item_id)
            .copied()
            .unwrap_or(0);

        if available_quantity < ingredient.quantity {
            return false;
        }
    }
    true
}

/// Update cooking machines and complete cooking when timer finishes
fn update_cooking_machines(
    mut commands: Commands,
    time: Res<Time>,
    mut q_machines: Query<(Entity, &mut Machine)>,
    recipe_registry: RecipeRegistry,
) {
    for (machine_entity, mut machine) in q_machines.iter_mut() {
        if matches!(machine.state, MachineState::Occupied) {
            machine.elapsed_time += time.delta_secs();

            if machine.is_cooking_done(&recipe_registry) {
                info!(
                    "Machine {:?} finished cooking recipe '{}'!",
                    machine_entity, machine.recipe_id
                );

                commands.trigger_targets(
                    TowerProductionEvent,
                    machine_entity,
                );
                machine.complete_cooking();
            }
        }
    }
}

/// Handles machine interaction, consumes ingredients and starts cooking
fn process_machine_interaction(
    trigger: Trigger<MachineInteractionEvent>,
    mut q_machines: Query<&mut Machine>,
    mut q_inventories: Query<&mut Inventory>,
    recipe_registry: RecipeRegistry,
) {
    let event = trigger.event();
    let machine_entity = trigger.target();
    let player_entity = event.player;

    let Ok(mut machine) = q_machines.get_mut(machine_entity) else {
        warn!("Machine entity {:?} not found", machine_entity);
        return;
    };

    let Ok(mut inventory) = q_inventories.get_mut(player_entity)
    else {
        warn!("Player entity {:?} has no inventory", player_entity);
        return;
    };

    let Some(recipe) = machine.get_recipe(&recipe_registry) else {
        warn!("Recipe '{}' not found in registry", machine.recipe_id);
        return;
    };

    // Double-check ingredients
    if !can_start_cooking(&inventory, &machine, &recipe_registry) {
        warn!(
            "Player {:?} no longer has required ingredients for recipe '{}'",
            player_entity, machine.recipe_id
        );
        return;
    }

    // Consume ingredients from inventory
    for ingredient in &recipe.ingredients {
        let current_quantity = inventory
            .get_all_ingredients()
            .get(&ingredient.item_id)
            .copied()
            .unwrap_or(0);

        let remaining_quantity =
            current_quantity - ingredient.quantity;

        if remaining_quantity > 0 {
            inventory.ingredients.insert(
                ingredient.item_id.clone(),
                remaining_quantity,
            );
        } else {
            inventory.ingredients.remove(&ingredient.item_id);
        }

        info!(
            "Consumed {}x {} from player {:?} for recipe '{}'",
            ingredient.quantity,
            ingredient.item_id,
            player_entity,
            machine.recipe_id
        );
    }

    // Start cooking
    machine.start_cooking();
    info!(
        "Machine {:?} started cooking recipe '{}' - will produce {}x {} in {:.1}s",
        machine_entity,
        machine.recipe_id,
        recipe.output_quantity,
        recipe.output_id,
        recipe.cooking_duration
    );
}

/// Observer that handles tower production when cooking completes
fn handle_tower_production(
    trigger: Trigger<TowerProductionEvent>,
    mut _commands: Commands,
    q_machines: Query<&Machine>,
    recipe_registry: RecipeRegistry,
) {
    let machine_entity = trigger.target();

    let Ok(machine) = q_machines.get(machine_entity) else {
        warn!("Machine entity {:?} not found", machine_entity);
        return;
    };

    let Some(recipe) = machine.get_recipe(&recipe_registry) else {
        warn!("Recipe '{}' not found in registry", machine.recipe_id);
        return;
    };

    // Spawn tower item in the world near the machine
    info!(
        "Machine {:?} produced {}x {} using recipe '{}'",
        machine_entity,
        recipe.output_quantity,
        recipe.output_id,
        machine.recipe_id
    );

    // TODO: Spawn actual tower item entity in the world
}

impl Machine {
    /// Get the recipe data from the registry
    pub fn get_recipe<'a>(
        &self,
        registry: &'a RecipeRegistry,
    ) -> Option<&'a RecipeMeta> {
        registry.get_recipe(&self.recipe_id)
    }

    pub fn start_cooking(&mut self) {
        self.state = MachineState::Occupied;
        self.elapsed_time = 0.0;
    }

    pub fn is_cooking_done(
        &self,
        recipe_registry: &RecipeRegistry,
    ) -> bool {
        if let Some(recipe) = self.get_recipe(recipe_registry) {
            matches!(self.state, MachineState::Occupied)
                && self.elapsed_time >= recipe.cooking_duration
        } else {
            false
        }
    }

    pub fn complete_cooking(&mut self) {
        self.state = MachineState::Ready;
        self.elapsed_time = 0.0;
    }

    pub fn cooking_progress(
        &self,
        recipe_registry: &RecipeRegistry,
    ) -> f32 {
        if let Some(recipe) = self.get_recipe(recipe_registry) {
            if recipe.cooking_duration <= 0.0 {
                return 1.0;
            }
            (self.elapsed_time / recipe.cooking_duration)
                .clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    pub fn remaining_time(
        &self,
        recipe_registry: &RecipeRegistry,
    ) -> f32 {
        if let Some(recipe) = self.get_recipe(recipe_registry) {
            (recipe.cooking_duration - self.elapsed_time).max(0.0)
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum MachineState {
    Ready,
    Occupied,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct MachineInteractionEvent {
    pub player: Entity,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct TowerProductionEvent;

/// Component representing a machine that can convert ingredients to towers
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Machine {
    pub state: MachineState,
    /// The ID of the recipe to use from the registry
    pub recipe_id: String,
    pub elapsed_time: f32,
}
