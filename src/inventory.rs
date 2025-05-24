use crate::interaction::InteractionPlayer;
use avian3d::prelude::*;
use bevy::prelude::*;

mod inventory_input;
mod item;

pub use item::ItemRegistry;

pub(super) struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            inventory_input::InventoryInputPlugin,
            item::ItemPlugin,
        ))
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
            }
        }
    }
}

/// Observer that triggers when ItemOf component is removed
fn handle_item_removed_from_inventory(
    trigger: Trigger<OnRemove, ItemOf>,
    mut q_inventories: Query<&mut Inventory>,
) {
    let item_entity = trigger.target();

    // Since we can't access the removed component's data directly,
    // we need to search through all inventories to find and remove the item
    for mut inventory in q_inventories.iter_mut() {
        if let Some(pos) =
            inventory.0.iter().position(|&e| e == item_entity)
        {
            inventory.0.remove(pos);
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
    item_registry: Res<ItemRegistry>,
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
        // Early return since we just inserted the inventory and need to wait for next frame
        commands.entity(item_entity).insert(ItemOf(player_entity));
        commands
            .entity(item_entity)
            .remove::<Pickupable>()
            .insert(RigidBodyDisabled)
            .insert(Visibility::Hidden);
        return;
    }

    // Get the item being picked up
    let Ok(item) = q_items.get(item_entity) else {
        warn!(
            "Attempted to pickup non-existent item: {:?}",
            item_entity
        );
        return;
    };

    // Get item metadata to check if it's stackable
    let Some(item_meta) = item_registry.by_id.get(&item.id) else {
        warn!("Item {} not found in registry", item.id);
        // Still allow pickup, just treat as non-stackable
        commands.entity(item_entity).insert(ItemOf(player_entity));
        commands
            .entity(item_entity)
            .remove::<Pickupable>()
            .insert(RigidBodyDisabled)
            .insert(Visibility::Hidden);
        return;
    };

    let mut item_consumed = false;
    let item_id = item.id;
    let item_quantity = item.quantity;

    // Try to stack with existing items if possible
    if item_meta.stackable {
        // Get the inventory to find existing items
        if let Ok(inventory) = q_inventories.get(player_entity) {
            // Collect entities that might be stackable
            let stackable_candidates: Vec<Entity> = inventory
                .0
                .iter()
                .copied()
                .filter(|&e| {
                    if let Ok(existing_item) = q_items.get(e) {
                        existing_item.id == item_id
                            && existing_item.quantity
                                < item_meta.max_stack_size
                    } else {
                        false
                    }
                })
                .collect();

            // Try to stack with the first suitable item
            if let Some(&target_entity) = stackable_candidates.first()
            {
                if let Ok(mut existing_item) =
                    q_items.get_mut(target_entity)
                {
                    let space_available = item_meta.max_stack_size
                        - existing_item.quantity;
                    let amount_to_add =
                        item_quantity.min(space_available);

                    // Add to existing stack
                    existing_item.quantity += amount_to_add;

                    if amount_to_add == item_quantity {
                        // Entire stack was consumed, despawn the picked up item
                        commands.entity(item_entity).despawn();
                        item_consumed = true;
                        info!(
                            "Player {:?} stacked {} {} (total: {})",
                            player_entity,
                            amount_to_add,
                            item_meta.name,
                            existing_item.quantity
                        );
                    } else {
                        // Partial stack, reduce the picked up item's quantity
                        drop(existing_item); // Release the mutable borrow
                        if let Ok(mut picked_item) =
                            q_items.get_mut(item_entity)
                        {
                            picked_item.quantity -= amount_to_add;
                        }
                        info!(
                            "Player {:?} partially stacked {} {} (remaining: {})",
                            player_entity,
                            amount_to_add,
                            item_meta.name,
                            item_quantity - amount_to_add
                        );
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

        info!(
            "Player {:?} picked up {}x {}",
            player_entity, item_quantity, item_meta.name
        );
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
    item_registry: Res<ItemRegistry>,
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
        let item_name = item_registry
            .by_id
            .get(&item.id)
            .map(|meta| meta.name.as_str())
            .unwrap_or("Unknown Item");

        if item.quantity > 1 {
            // Reduce quantity
            item.quantity -= 1;
            info!(
                "Player {:?} consumed 1x {} (remaining: {})",
                player_entity, item_name, item.quantity
            );
        } else {
            // Remove item entirely
            commands.entity(item_entity).remove::<ItemOf>();
            commands.entity(item_entity).despawn();
            info!(
                "Player {:?} consumed last {}x {}",
                player_entity, item.quantity, item_name
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
    /// A unique identifier that corresponds to ItemMeta
    pub id: u32,
    /// How many are in this stack
    pub quantity: u32,
}
