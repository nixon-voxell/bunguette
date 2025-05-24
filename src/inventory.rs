use crate::interaction::InteractionPlayer;
use avian3d::prelude::*;
use bevy::prelude::*;

mod inventory_input;
pub(super) struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(inventory_input::InventoryInputPlugin)
            .add_observer(handle_item_added_to_inventory)
            .add_observer(handle_item_removed_from_inventory)
            .add_observer(handle_pickup)
            .add_observer(handle_drop)
            .add_observer(handle_consume);

        app.register_type::<Inventory>()
            .register_type::<Pickupable>()
            .register_type::<Consumable>()
            .register_type::<Item>();
    }
}

/// Observer that triggers when ItemOf component is added
fn handle_item_added_to_inventory(
    trigger: Trigger<OnAdd, ItemOf>,
    mut q_inventories: Query<&mut Inventory>,
    q_items: Query<&Item>,
    q_item_of: Query<&ItemOf>,
) {
    let item_entity = trigger.target();

    // Get the ItemOf component from the entity that just had it added
    if let Ok(item_of) = q_item_of.get(item_entity) {
        let player_entity = item_of.0;

        if let Ok(mut inventory) =
            q_inventories.get_mut(player_entity)
        {
            if !inventory.0.contains(&item_entity) {
                inventory.0.push(item_entity);

                if let Ok(item) = q_items.get(item_entity) {
                    info!(
                        "Added '{}' to player {:?}'s inventory",
                        item.name, player_entity
                    );
                }
            }
        }
    }
}

/// Observer that triggers when ItemOf component is removed
fn handle_item_removed_from_inventory(
    trigger: Trigger<OnRemove, ItemOf>,
    mut q_inventories: Query<&mut Inventory>,
    q_items: Query<&Item>,
) {
    let item_entity = trigger.target();

    // Since we can't access the removed component's data directly,
    // we need to search through all inventories to find and remove the item
    for mut inventory in q_inventories.iter_mut() {
        if let Some(pos) =
            inventory.0.iter().position(|&e| e == item_entity)
        {
            inventory.0.remove(pos);

            if let Ok(item) = q_items.get(item_entity) {
                info!("Removed '{}' from inventory", item.name);
            }
            break;
        }
    }
}

/// Handles pickup events - adds items to inventory
fn handle_pickup(
    trigger: Trigger<PickupEvent>,
    mut commands: Commands,
    q_inventories: Query<&mut Inventory>,
    mut q_items: Query<&mut Item>,
    q_players: Query<Entity, With<InteractionPlayer>>,
) {
    let player_entity = trigger.target();
    let item_entity = trigger.event().item;

    // Verify this is actually a player entity
    if q_players.get(player_entity).is_err() {
        warn!(
            "Attempted to pickup item for non-player entity: {:?}",
            player_entity
        );
        return;
    }

    // Ensure player has an inventory
    if q_inventories.get(player_entity).is_err() {
        commands.entity(player_entity).insert(Inventory::default());
    }

    // Get the item being picked up
    let Ok(item) = q_items.get(item_entity) else {
        warn!(
            "Attempted to pickup non-existent item: {:?}",
            item_entity
        );
        return;
    };

    let mut item_consumed = false;

    // Try to stack with existing items if possible
    if item.stackable {
        if let Ok(inventory) = q_inventories.get(player_entity) {
            let item_id = item.id;
            let item_quantity = item.quantity;

            // Look for existing stackable items of the same type
            for &existing_item in inventory.0.iter() {
                if let Ok(mut existing_item_data) =
                    q_items.get_mut(existing_item)
                {
                    if existing_item_data.id == item_id
                        && existing_item_data.quantity
                            < existing_item_data.max_stack_size
                    {
                        let space_available = existing_item_data
                            .max_stack_size
                            - existing_item_data.quantity;
                        let amount_to_add =
                            item_quantity.min(space_available);

                        // Add to existing stack
                        existing_item_data.quantity += amount_to_add;

                        if amount_to_add == item_quantity {
                            // Entire stack was consumed, despawn the picked up item
                            commands.entity(item_entity).despawn();
                            item_consumed = true;

                            info!(
                                "Player {:?} picked up {}x {} (stacked)",
                                player_entity,
                                amount_to_add,
                                existing_item_data.name
                            );
                        } else {
                            // Partial stack, reduce the picked up item's quantity
                            let mut reduce_quantity = false;
                            if amount_to_add < item_quantity {
                                reduce_quantity = true;
                            }

                            info!(
                                "Player {:?} picked up {}x {} (partial stack, {} remaining)",
                                player_entity,
                                amount_to_add,
                                existing_item_data.name,
                                item_quantity - amount_to_add
                            );

                            // Reduce the picked up item's quantity after the previous mutable borrow ends
                            if reduce_quantity {
                                if let Ok(mut picked_item) =
                                    q_items.get_mut(item_entity)
                                {
                                    picked_item.quantity -=
                                        amount_to_add;
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    // If item wasn't fully consumed by stacking, add it as a new inventory item
    if !item_consumed {
        // Add item to inventory relationship
        commands.entity(item_entity).insert(ItemOf(player_entity));

        // Remove from world (disable physics, hide mesh, etc.)
        commands
            .entity(item_entity)
            .remove::<Pickupable>()
            .insert(RigidBodyDisabled)
            .insert(Visibility::Hidden);

        if let Ok(item) = q_items.get(item_entity) {
            info!(
                "Player {:?} picked up {}x {} (new item)",
                player_entity, item.quantity, item.name
            );
        }
    }
}

/// Handles drop events - removes items from inventory and places them in world
fn handle_drop(
    trigger: Trigger<DropEvent>,
    mut commands: Commands,
    q_player_transforms: Query<
        &GlobalTransform,
        With<InteractionPlayer>,
    >,
    mut q_item_transforms: Query<&mut Transform>,
    q_players: Query<Entity, With<InteractionPlayer>>,
) {
    let player_entity = trigger.target();
    let item_entity = trigger.event().item;

    // Verify this is actually a player entity
    if q_players.get(player_entity).is_err() {
        warn!(
            "Attempted to drop item for non-player entity: {:?}",
            player_entity
        );
        return;
    }

    // Remove from inventory relationship
    commands.entity(item_entity).remove::<ItemOf>();

    // Place item in world in front of player
    if let Ok(player_transform) =
        q_player_transforms.get(player_entity)
    {
        if let Ok(mut item_transform) =
            q_item_transforms.get_mut(item_entity)
        {
            let drop_position = player_transform.translation()
                + player_transform.forward() * 2.0;
            item_transform.translation = drop_position;
        }
    }

    // Re-enable physics and visibility
    commands
        .entity(item_entity)
        .insert(Pickupable)
        .remove::<RigidBodyDisabled>()
        .insert(Visibility::Visible);

    info!(
        "Player {:?} dropped item {:?}",
        player_entity, item_entity
    );
}

/// Handles consume events - removes consumable items from inventory
fn handle_consume(
    trigger: Trigger<ConsumeEvent>,
    mut commands: Commands,
    mut q_items: Query<&mut Item>,
    q_players: Query<Entity, With<InteractionPlayer>>,
) {
    let player_entity = trigger.target();
    let item_entity = trigger.event().item;

    // Verify this is actually a player entity
    if q_players.get(player_entity).is_err() {
        warn!(
            "Attempted to consume item for non-player entity: {:?}",
            player_entity
        );
        return;
    }

    // Handle consumption logic
    if let Ok(mut item) = q_items.get_mut(item_entity) {
        if item.quantity > 1 {
            // Reduce quantity
            item.quantity -= 1;
            info!(
                "Player {:?} consumed 1x {}, {} remaining",
                player_entity, item.name, item.quantity
            );
        } else {
            // Remove item entirely
            commands.entity(item_entity).remove::<ItemOf>();
            commands.entity(item_entity).despawn();
            info!(
                "Player {:?} consumed last {}",
                player_entity, item.name
            );
        }
    }

    // TODO: Apply consumption effects (heal player, give buff, etc.)
}

#[derive(Event)]
pub struct PickupEvent {
    pub item: Entity,
}

#[derive(Event)]
pub struct DropEvent {
    pub item: Entity,
}

#[derive(Event)]
pub struct ConsumeEvent {
    pub item: Entity,
}

/// Relationship component that marks an entity as belonging to an inventory
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ItemOf(pub Entity);

/// Marks an entity as having an inventory
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Inventory(pub Vec<Entity>);

/// Marks an item as pickupable from the world
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Pickupable;

/// Tag for consumable items
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Consumable;

/// Core data for any inventory item.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Item {
    /// A unique identifier
    pub id: u32,
    /// The display name shown in UI/tooltips.
    pub name: String,
    /// Icon or sprite handle for UI rendering.
    pub icon: Option<Handle<Image>>,
    /// A longer description text.
    pub description: Option<String>,
    /// Can multiple of these stack in one slot?
    pub stackable: bool,
    /// How many are in this stack (only used if `stackable == true`).
    pub quantity: u32,
    /// Maximum stack size (only relevant if stackable is true)
    pub max_stack_size: u32,
}
