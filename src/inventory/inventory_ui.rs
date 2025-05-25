use super::{Inventory, Item, ItemRegistry};
use crate::interaction::InteractionPlayer;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

pub struct InventoryUiPlugin;

impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                toggle_inventory_system,
                update_inventory_ui_system
                    .run_if(resource_exists::<InventoryUiState>),
                debug_inventory_ui_system,
            ),
        )
        .init_resource::<InventoryUiState>();
    }
}

/// Toggle inventory UI with Tab key - works for local player or first player found
fn toggle_inventory_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<InventoryUiState>,
    // Try local player first, then any player
    q_local_player: Query<
        Entity,
        (With<InteractionPlayer>, With<LocalPlayer>),
    >,
    q_any_player: Query<Entity, With<InteractionPlayer>>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        // Determine which player to show inventory for
        let target_player =
            if let Ok(local_player) = q_local_player.single() {
                Some(local_player)
            } else if let Ok(any_player) = q_any_player.single() {
                Some(any_player)
            } else {
                None
            };

        let Some(player_entity) = target_player else {
            warn!("No player found to show inventory for");
            return;
        };

        // Toggle inventory for this specific player
        if ui_state.open_for_player == Some(player_entity) {
            // Close inventory
            if let Some(ui_entity) = ui_state.ui_entity {
                commands.entity(ui_entity).despawn();
            }
            ui_state.open_for_player = None;
            ui_state.ui_entity = None;
        } else {
            // Close any existing inventory UI first
            if let Some(ui_entity) = ui_state.ui_entity {
                commands.entity(ui_entity).despawn();
            }

            // Open inventory for this player
            let ui_entity = spawn_inventory_ui(&mut commands);
            ui_state.open_for_player = Some(player_entity);
            ui_state.ui_entity = Some(ui_entity);
        }
    }
}

/// Update inventory UI contents for the specific player whose inventory is open
fn update_inventory_ui_system(
    ui_state: Res<InventoryUiState>,
    q_inventories: Query<&Inventory, With<InteractionPlayer>>,
    q_items: Query<&Item>,
    item_registry: Res<ItemRegistry>,
    mut q_slots: Query<(&InventorySlot, &Children)>,
    mut q_images: Query<&mut ImageNode>,
    mut q_item_name_text: Query<
        &mut Text,
        (With<ItemNameText>, Without<QuantityText>),
    >,
    mut q_quantity_text: Query<
        &mut Text,
        (With<QuantityText>, Without<ItemNameText>),
    >,
) {
    // Only update if UI is open for a specific player
    let Some(player_entity) = ui_state.open_for_player else {
        return;
    };

    let Some(_ui_entity) = ui_state.ui_entity else {
        return;
    };

    // Get the specific player's inventory
    let Ok(inventory) = q_inventories.get(player_entity) else {
        return;
    };

    // Update each slot for this player's inventory
    for (slot, children) in q_slots.iter_mut() {
        let item_entity = inventory.0.get(slot.slot_index);

        // Update slot contents based on whether there's an item
        if let Some(&item_entity) = item_entity {
            if let Ok(item) = q_items.get(item_entity) {
                let item_meta = item_registry.by_id.get(&item.id);
                let item_name = item_meta
                    .map(|m| m.name.as_str())
                    .unwrap_or("Unknown");

                // Update children components
                for child in children.iter() {
                    // Try to update image
                    if let Ok(mut image_node) =
                        q_images.get_mut(child)
                    {
                        if let Some(icon_handle) =
                            item_registry.icons.get(&item.id)
                        {
                            image_node.image = icon_handle.clone();
                        } else {
                            image_node.image = Handle::default();
                        }
                    }

                    // Try to update item name text (fallback when no icon)
                    if let Ok(mut text) =
                        q_item_name_text.get_mut(child)
                    {
                        if item_registry.icons.get(&item.id).is_none()
                        {
                            text.0 = item_name.to_string();
                        } else {
                            text.0 = String::new();
                        }
                    }

                    // Try to update quantity text
                    if let Ok(mut text) =
                        q_quantity_text.get_mut(child)
                    {
                        if item.quantity > 1 {
                            text.0 = item.quantity.to_string();
                        } else {
                            text.0 = String::new();
                        }
                    }
                }
            }
        } else {
            // Clear empty slot content
            for child in children.iter() {
                if let Ok(mut image_node) = q_images.get_mut(child) {
                    image_node.image = Handle::default();
                }
                if let Ok(mut text) = q_item_name_text.get_mut(child)
                {
                    text.0 = String::new();
                }
                if let Ok(mut text) = q_quantity_text.get_mut(child) {
                    text.0 = String::new();
                }
            }
        }
    }
}

/// Debug system - shows inventory for all players
fn debug_inventory_ui_system(
    keys: Res<ButtonInput<KeyCode>>,
    ui_state: Res<InventoryUiState>,
    q_players: Query<(Entity, &Inventory), With<InteractionPlayer>>,
    q_items: Query<&Item>,
    item_registry: Res<ItemRegistry>,
    q_slots: Query<&InventorySlot>,
) {
    if keys.just_pressed(KeyCode::F1) {
        info!("=== MULTIPLAYER INVENTORY DEBUG ===");
        info!("UI Open for player: {:?}", ui_state.open_for_player);
        info!("UI Entity: {:?}", ui_state.ui_entity);

        let player_count = q_players.iter().count();
        info!("Found {} players with inventories:", player_count);

        for (player_entity, inventory) in q_players.iter() {
            info!(
                "Player {:?} inventory ({} items):",
                player_entity,
                inventory.0.len()
            );
            for (i, &item_entity) in inventory.0.iter().enumerate() {
                if let Ok(item) = q_items.get(item_entity) {
                    let item_name = item_registry
                        .by_id
                        .get(&item.id)
                        .map(|meta| meta.name.as_str())
                        .unwrap_or("Unknown");
                    info!(
                        "  Slot {}: {}x {} (Entity: {:?})",
                        i, item.quantity, item_name, item_entity
                    );
                }
            }
        }

        let slot_count = q_slots.iter().count();
        info!("UI has {} slots created", slot_count);
        info!("=================================");
    }
}

fn spawn_inventory_ui(commands: &mut Commands) -> Entity {
    const SLOT_SIZE: f32 = 64.0;
    const GRID_COLS: usize = 8;
    const GRID_ROWS: usize = 4;
    const SLOT_GAP: f32 = 4.0;
    const PANEL_PADDING: f32 = 16.0;

    let total_width = (SLOT_SIZE * GRID_COLS as f32)
        + (SLOT_GAP * (GRID_COLS - 1) as f32)
        + (PANEL_PADDING * 2.0);
    let total_height = (SLOT_SIZE * GRID_ROWS as f32)
        + (SLOT_GAP * (GRID_ROWS - 1) as f32)
        + (PANEL_PADDING * 2.0);

    commands
        .spawn((
            InventoryUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(total_width),
                height: Val::Px(total_height),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px(
                    GRID_COLS as u16,
                    SLOT_SIZE,
                ),
                grid_template_rows: RepeatedGridTrack::px(
                    GRID_ROWS as u16,
                    SLOT_SIZE,
                ),
                column_gap: Val::Px(SLOT_GAP),
                row_gap: Val::Px(SLOT_GAP),
                padding: UiRect::all(Val::Px(PANEL_PADDING)),
                border: UiRect::all(Val::Px(2.0)),
                // Center the inventory panel
                margin: UiRect {
                    left: Val::Px(-total_width / 2.0),
                    top: Val::Px(-total_height / 2.0),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
            FocusPolicy::Block,
        ))
        .with_children(|parent| {
            // Create inventory slots
            for slot_index in 0..(GRID_COLS * GRID_ROWS) {
                parent
                    .spawn((
                        InventorySlot { slot_index },
                        // BackgroundColor(Color::NONE),
                        Node {
                            width: Val::Px(SLOT_SIZE),
                            height: Val::Px(SLOT_SIZE),
                            border: UiRect::all(Val::Px(1.0)),
                            position_type: PositionType::Relative,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(
                            0.2, 0.2, 0.2, 0.8,
                        )),
                        BorderColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                    ))
                    .with_children(|slot_parent| {
                        // Item icon
                        slot_parent.spawn((
                            ImageNode {
                                color: Color::NONE,
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            Node {
                                width: Val::Percent(90.0),
                                height: Val::Percent(90.0),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                        ));

                        // Item name text (fallback when no icon)
                        slot_parent.spawn((
                            ItemNameText,
                            Text::new(""),
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                justify_content:
                                    JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            TextFont {
                                font_size: 10.0,
                                ..default()
                            },
                        ));

                        // Quantity text (bottom-right corner)
                        slot_parent.spawn((
                            QuantityText,
                            Text::new(""),
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(2.0),
                                right: Val::Px(2.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                        ));
                    });
            }
        })
        .id()
}

#[derive(Resource, Default)]
struct InventoryUiState {
    // Track which player's inventory is currently open
    open_for_player: Option<Entity>,
    ui_entity: Option<Entity>,
}

#[derive(Component)]
struct InventoryUiRoot;

#[derive(Component)]
struct InventorySlot {
    slot_index: usize,
}

#[derive(Component)]
struct ItemNameText;

#[derive(Component)]
struct QuantityText;

// Add a component to identify the local player (for single-screen multiplayer)
// or use networking player ID for networked multiplayer
// TODO: Implement proper multiplayer player identification
#[derive(Component)]
pub struct LocalPlayer;
