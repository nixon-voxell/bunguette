use crate::character_controller::CharacterController;
use crate::physics::GameLayer;
use avian3d::prelude::*;
use bevy::prelude::*;
use item::{ItemRegistry, ItemType};

mod inventory_input;
pub mod item;

pub(super) struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            inventory_input::InventoryInputPlugin,
            item::ItemPlugin,
        ))
        .add_observer(setup_item_collision)
        .add_observer(handle_item_collection)
        .add_systems(Update, detect_item_collisions);

        app.register_type::<Inventory>().register_type::<Item>();
    }
}

fn setup_item_collision(
    trigger: Trigger<OnAdd, Item>,
    mut commands: Commands,
) {
    commands.entity(trigger.target()).insert((
        CollisionLayers::new(
            GameLayer::InventoryItem,
            LayerMask::ALL,
        ),
        CollidingEntities::default(),
    ));
}

/// Detect item collection
fn detect_item_collisions(
    mut commands: Commands,
    q_players: Query<
        (Entity, &CollidingEntities),
        With<CharacterController>,
    >,
    q_items: Query<&Item>,
    item_registry: ItemRegistry,
) {
    let Some(item_meta_asset) = item_registry.get() else {
        return;
    };

    // Check each player's colliding entities
    for (player_entity, colliding_entities) in q_players.iter() {
        // Check all entities currently colliding with this player
        for &colliding_entity in colliding_entities.iter() {
            // Check if the colliding entity is an item
            if let Ok(item) = q_items.get(colliding_entity) {
                if let Some(item_meta) = item_meta_asset.get(&item.id)
                {
                    // Only auto-collect ingredients
                    if item_meta.item_type == ItemType::Ingredient {
                        info!(
                            "Player {:?} collecting item {:?} via CollidingEntities",
                            player_entity, colliding_entity
                        );

                        // Trigger collection event
                        commands.trigger_targets(
                            ItemCollectionEvent {
                                item: colliding_entity,
                            },
                            player_entity,
                        );
                    }
                }
            }
        }
    }
}

/// Observer that handles item collection
fn handle_item_collection(
    trigger: Trigger<ItemCollectionEvent>,
    mut commands: Commands,
    mut q_inventories: Query<&mut Inventory>,
    q_items: Query<&Item>,
    q_players: Query<Entity, With<CharacterController>>,
    item_registry: ItemRegistry,
) {
    let Some(item_meta_asset) = item_registry.get() else {
        return;
    };

    let player_entity = trigger.target();
    let item_entity = trigger.event().item;

    if q_players.get(player_entity).is_err() {
        warn!(
            "Attempted to collect item for non-player entity: {:?}",
            player_entity
        );
        return;
    }

    // Get the item being collected
    let Ok(world_item) = q_items.get(item_entity) else {
        warn!(
            "Attempted to collect non-existent item: {:?}",
            item_entity
        );
        return;
    };

    let Some(item_meta) = item_meta_asset.get(&world_item.id) else {
        warn!("Item {} not found in registry", world_item.id);
        return;
    };

    // Ensure player has an inventory
    let mut inventory_just_created = false;
    if q_inventories.get(player_entity).is_err() {
        commands.entity(player_entity).insert(Inventory::default());
        inventory_just_created = true;
        info!("Created new inventory for player {:?}", player_entity);
    }

    if inventory_just_created {
        commands.trigger_targets(
            ItemCollectionEvent { item: item_entity },
            player_entity,
        );
        return;
    }

    let Ok(mut inventory) = q_inventories.get_mut(player_entity)
    else {
        warn!("Player {:?} has no inventory", player_entity);
        return;
    };

    let item_id = &world_item.id;
    let collected_quantity = world_item.quantity;

    // Add to inventory based on item type
    let success = match item_meta.item_type {
        ItemType::Ingredient => inventory.add_ingredient(
            item_id.clone(),
            collected_quantity,
            item_meta.max_stack_size,
        ),
        ItemType::Tower => inventory.add_tower(
            item_id.clone(),
            collected_quantity,
            item_meta.max_stack_size,
        ),
    };

    if success {
        info!(
            "Player {:?} collected {}x {} ({})",
            player_entity,
            collected_quantity,
            item_id,
            match item_meta.item_type {
                ItemType::Ingredient => "ingredient",
                ItemType::Tower => "tower",
            }
        );

        // Remove the item from the world
        commands.entity(item_entity).despawn();
    } else {
        // TODO: Handle stack overflow
        // For now, just log a warning
        warn!(
            "Could not collect {}x {}: would exceed max stack size ({})",
            collected_quantity, item_id, item_meta.max_stack_size
        );
    }
}

impl Inventory {
    /// Add towers to the inventory with stack limit checking
    pub fn add_tower(
        &mut self,
        tower_id: String,
        quantity: u32,
        max_stack_size: u32,
    ) -> bool {
        let current_count =
            self.towers.get(&tower_id).copied().unwrap_or(0);
        let new_total = current_count + quantity;

        if new_total <= max_stack_size {
            self.towers.insert(tower_id, new_total);
            true
        } else {
            false
        }
    }

    /// Add ingredients to the inventory with stack limit checking
    pub fn add_ingredient(
        &mut self,
        ingredient_id: String,
        quantity: u32,
        max_stack_size: u32,
    ) -> bool {
        let current_count = self
            .ingredients
            .get(&ingredient_id)
            .copied()
            .unwrap_or(0);
        let new_total = current_count + quantity;

        if new_total <= max_stack_size {
            self.ingredients.insert(ingredient_id, new_total);
            true
        } else {
            false
        }
    }

    /// Get all ingredients for display
    pub fn get_all_ingredients(
        &self,
    ) -> &std::collections::HashMap<String, u32> {
        &self.ingredients
    }
}

#[derive(Event)]
pub struct ItemCollectionEvent {
    pub item: Entity,
}

/// Marks an entity as having an inventory for both towers and ingredients
#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
pub struct Inventory {
    /// Map of tower ID to quantity available (can be selected and placed)
    pub towers: std::collections::HashMap<String, u32>,
    /// Map of ingredient ID to quantity collected (display only, cannot be selected)
    pub ingredients: std::collections::HashMap<String, u32>,
    /// Currently selected tower for placement (if any)
    pub selected_tower: Option<String>,
}

/// Core data for any item (both towers and ingredients).
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Item {
    /// A unique identifier that corresponds to [`item::ItemMeta`]
    pub id: String,
    /// How many are in this stack.
    pub quantity: u32,
}
