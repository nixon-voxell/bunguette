use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use recipe::RecipeMeta;

use crate::action::{PlayerAction, TargetAction};
use crate::interaction::MarkerOf;
use crate::inventory::Inventory;
use crate::inventory::item::ItemRegistry;
use crate::machine::recipe::RecipeRegistry;

mod animation;
mod machine_ui;
pub mod recipe;

pub(super) struct MachinePlugin;

impl Plugin for MachinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            machine_ui::MachineUiPlugin,
            recipe::RecipePlugin,
            animation::MachineAnimationPlugin,
        ))
        .add_systems(Update, handle_player_machine_interaction)
        .add_systems(Update, update_cooking_machines);
    }
}

/// Handle player interaction with machines
fn handle_player_machine_interaction(
    mut commands: Commands,
    mut q_players: Query<(
        &MarkerOf,
        &TargetAction,
        &mut Inventory,
        Entity,
    )>,
    q_actions: Query<&ActionState<PlayerAction>>,
    // Get only non-operating machines.
    q_machines: Query<&Machine, Without<OperatedBy>>,
    recipe_registry: RecipeRegistry,
) {
    for (marked_item, target_action, mut inventory, player_entity) in
        q_players.iter_mut()
    {
        let machine_entity = marked_item.entity();
        let Ok(machine) = q_machines.get(machine_entity) else {
            continue;
        };

        let Ok(action_state) = q_actions.get(target_action.get())
        else {
            continue;
        };

        if !action_state.just_pressed(&PlayerAction::Interact) {
            continue;
        };

        let Some(recipe) =
            recipe_registry.get_recipe(&machine.recipe_id)
        else {
            warn!(
                "Recipe '{}' not found in registry",
                machine.recipe_id
            );
            continue;
        };

        if inventory.check_and_use_recipe(recipe) {
            commands.entity(machine_entity).insert((
                OperationTimer(Timer::from_seconds(
                    recipe.cooking_duration,
                    TimerMode::Once,
                )),
                OperatedBy(player_entity),
            ));
        } else {
            info!(
                "Player {} doesn't have required ingredients for recipe '{}'",
                player_entity, machine.recipe_id
            );
        }
    }
}

/// Update cooking machines and complete cooking when timer finishes.
fn update_cooking_machines(
    mut commands: Commands,
    mut q_machines: Query<(
        &Machine,
        &mut OperationTimer,
        &OperatedBy,
        Entity,
    )>,
    mut q_inventories: Query<&mut Inventory>,
    recipe_registry: RecipeRegistry,
    item_registry: ItemRegistry,
    time: Res<Time>,
) {
    for (machine, mut timer, operated_by, entity) in
        q_machines.iter_mut()
    {
        if timer.tick(time.delta()).finished() == false {
            continue;
        }

        let Some(recipe) =
            recipe_registry.get_recipe(&machine.recipe_id)
        else {
            warn!(
                "Recipe '{}' not found in registry",
                machine.recipe_id
            );
            continue;
        };

        let Some(item) = item_registry.get_item(&recipe.output_id)
        else {
            warn!(
                "Output item '{}' not found in item registry",
                recipe.output_id
            );
            continue;
        };

        commands
            .entity(entity)
            .remove::<(OperationTimer, OperatedBy)>();

        let player_entity = operated_by.entity();
        if let Ok(mut inventory) =
            q_inventories.get_mut(player_entity)
        {
            // Add tower to player's inventory
            inventory.add_tower(
                recipe.output_id.clone(),
                recipe.output_quantity,
                // TODO: Handle when stack size exceeds!
                // Should not happen in the first place anyways...
                // Could happen if there are more than 1 similar machines...
                item.max_stack_size,
            );
        } else {
            error!(
                "Could not get inventory for player {}",
                player_entity
            );
        }
    }
}

/// Component representing a machine that can convert ingredients to towers
#[derive(Component, Reflect, Debug, Clone)]
#[component(immutable)]
#[reflect(Component)]
pub struct Machine {
    /// The ID of the recipe to use from the registry
    pub recipe_id: String,
}

impl Machine {
    /// Get the recipe data from the registry
    pub fn get_recipe<'a>(
        &self,
        registry: &'a RecipeRegistry,
    ) -> Option<&'a RecipeMeta> {
        registry.get_recipe(&self.recipe_id)
    }

    pub fn get_icon(
        &self,
        recipe_registry: &RecipeRegistry,
        item_registry: &ItemRegistry,
    ) -> Option<Handle<Image>> {
        let recipe = self.get_recipe(recipe_registry)?;
        let item = item_registry.get_item(&recipe.output_id)?;

        Some(item.icon.clone())
    }
}

#[derive(Component, Deref, Default, Debug)]
#[relationship_target(relationship = OperatedBy)]
pub struct OperatingMachines(Vec<Entity>);

#[derive(Component, Deref, Debug)]
#[relationship(relationship_target = OperatingMachines)]
pub struct OperatedBy(Entity);

#[derive(Component, Deref, DerefMut)]
pub struct OperationTimer(Timer);
