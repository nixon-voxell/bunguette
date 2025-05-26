use super::{Inventory, Item, ItemRegistry};
use crate::interaction::InteractionPlayer;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

pub struct InventoryUiPlugin;

impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_selected_item_ui)
            .add_systems(
                Update,
                (
                    toggle_inventory,
                    update_inventory_ui
                        .run_if(resource_exists::<InventoryUiState>),
                    update_selected_item_ui,
                    handle_slot_clicks,
                    debug_inventory_ui,
                ),
            )
            .init_resource::<InventoryUiState>()
            .init_resource::<SelectedItemUi>();
    }
}

/// Toggle inventory UI with Tab key - works for local player or first player found
fn toggle_inventory(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<InventoryUiState>,
    // Try local player first, then any player
    q_local_player: Query<
        Entity,
        (With<InteractionPlayer>, With<LocalPlayer>),
    >,
    q_any_player: Query<Entity, With<InteractionPlayer>>,
    q_inventories: Query<&Inventory, With<InteractionPlayer>>,
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

            if let Ok(inventory) = q_inventories.get(player_entity) {
                let ui_entity = spawn_inventory_ui(
                    &mut commands,
                    inventory.capacity,
                );
                ui_state.open_for_player = Some(player_entity);
                ui_state.ui_entity = Some(ui_entity);
            }
        }
    }
}

/// Handle clicking on inventory slots
fn handle_slot_clicks(
    mut q_inventories: Query<&mut Inventory, With<InteractionPlayer>>,
    ui_state: Res<InventoryUiState>,
    q_interactions: Query<
        (Entity, &Interaction),
        (Changed<Interaction>, With<InventorySlot>),
    >,
    q_slots: Query<&InventorySlot>,
) {
    // Only handle clicks if inventory UI is open
    let Some(player_entity) = ui_state.open_for_player else {
        return;
    };

    for (slot_entity, interaction) in q_interactions.iter() {
        if *interaction == Interaction::Pressed {
            if let Ok(slot) = q_slots.get(slot_entity) {
                if let Ok(mut inventory) =
                    q_inventories.get_mut(player_entity)
                {
                    if slot.slot_index < inventory.capacity {
                        inventory.selected_index =
                            Some(slot.slot_index);
                        info!(
                            "Clicked slot {} for player {:?}",
                            slot.slot_index + 1,
                            player_entity
                        );
                    }
                }
            }
        }
    }
}

/// Update inventory UI contents for the specific player whose inventory is open
fn update_inventory_ui(
    ui_state: Res<InventoryUiState>,
    q_inventories: Query<&Inventory, With<InteractionPlayer>>,
    q_items: Query<&Item>,
    item_registry: Res<ItemRegistry>,
    mut q_slots: Query<
        (&InventorySlot, &Children, &mut BackgroundColor),
        With<InventorySlot>,
    >,
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
    for (slot, children, mut background_color) in q_slots.iter_mut() {
        let item_entity = inventory.items.get(slot.slot_index);

        // Update slot background based on selection
        let is_selected =
            inventory.selected_index == Some(slot.slot_index);

        *background_color = if is_selected {
            BackgroundColor(Color::srgba(0.4, 0.4, 0.8, 0.9))
        } else if item_entity.is_some() {
            BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8))
        } else {
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8))
        };

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
fn debug_inventory_ui(
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
                "Player {:?} inventory ({} items, capacity: {}, selected: {:?}):",
                player_entity,
                inventory.items.len(),
                inventory.capacity,
                inventory.selected_index
            );
            for (i, &item_entity) in
                inventory.items.iter().enumerate()
            {
                if let Ok(item) = q_items.get(item_entity) {
                    let item_name = item_registry
                        .by_id
                        .get(&item.id)
                        .map(|meta| meta.name.as_str())
                        .unwrap_or("Unknown");

                    let selected_marker =
                        if inventory.selected_index == Some(i) {
                            " [SELECTED]"
                        } else {
                            ""
                        };

                    info!(
                        "  Slot {}: {}x {} (Entity: {:?}){}",
                        i,
                        item.quantity,
                        item_name,
                        item_entity,
                        selected_marker
                    );
                }
            }
        }

        let slot_count = q_slots.iter().count();
        info!("UI has {} slots created", slot_count);
        info!("=================================");
    }
}

fn spawn_inventory_ui(
    commands: &mut Commands,
    capacity: usize,
) -> Entity {
    const SLOT_SIZE: f32 = 64.0;
    const SLOT_GAP: f32 = 4.0;
    const PANEL_PADDING: f32 = 16.0;

    // Calculate grid dimensions based on capacity
    let grid_cols = (capacity as f32).sqrt().ceil() as usize;
    let grid_rows = (capacity + grid_cols - 1) / grid_cols;

    let total_width = (SLOT_SIZE * grid_cols as f32)
        + (SLOT_GAP * (grid_cols as f32 - 1.0))
        + (PANEL_PADDING * 2.0);
    let total_height = (SLOT_SIZE * grid_rows as f32)
        + (SLOT_GAP * (grid_rows as f32 - 1.0))
        + (PANEL_PADDING * 2.0)
        + 40.0;

    commands
        .spawn((
            InventoryUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(total_width),
                height: Val::Px(total_height),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(PANEL_PADDING)),
                border: UiRect::all(Val::Px(2.0)),
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
            // Instructions text
            parent.spawn((
                Text::new("Alt + Scroll select slots, Q to drop, C to consume"),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
            ));

            // Grid container for inventory slots
            parent
                .spawn((
                    Node {
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::px(
                            grid_cols as u16,
                            SLOT_SIZE,
                        ),
                        grid_template_rows: RepeatedGridTrack::px(
                            grid_rows as u16,
                            SLOT_SIZE,
                        ),
                        column_gap: Val::Px(SLOT_GAP),
                        row_gap: Val::Px(SLOT_GAP),
                        ..default()
                    },
                ))
                .with_children(|grid_parent| {
                    for slot_index in 0..capacity {
                        grid_parent
                            .spawn((
                                InventorySlot { slot_index },
                                Button,
                                Node {
                                    width: Val::Px(SLOT_SIZE),
                                    height: Val::Px(SLOT_SIZE),
                                    border: UiRect::all(Val::Px(1.0)),
                                    position_type: PositionType::Relative,
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                                BorderColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                            ))
                            .with_children(|slot_parent| {
                                // Slot number label (small text in corner)
                                slot_parent.spawn((
                                    Text::new((slot_index + 1).to_string()),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        top: Val::Px(2.0),
                                        left: Val::Px(2.0),
                                        ..default()
                                    },
                                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                                    TextFont {
                                        font_size: 8.0,
                                        ..default()
                                    },
                                ));

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
                                        justify_content: JustifyContent::Center,
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
                });
        })
        .id()
}

fn spawn_selected_item_ui(
    mut commands: Commands,
    mut selected_ui: ResMut<SelectedItemUi>,
) {
    const SLOT_SIZE: f32 = 48.0;
    const PANEL_PADDING: f32 = 8.0;

    let total_width = SLOT_SIZE + (PANEL_PADDING * 2.0) + 100.0;
    let total_height = SLOT_SIZE + (PANEL_PADDING * 2.0);

    let ui_entity = commands
        .spawn((
            SelectedItemUiRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                right: Val::Px(20.0),
                width: Val::Px(total_width),
                height: Val::Px(total_height),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(PANEL_PADDING)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
            FocusPolicy::Pass,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    SelectedItemSlot,
                    Node {
                        width: Val::Px(SLOT_SIZE),
                        height: Val::Px(SLOT_SIZE),
                        border: UiRect::all(Val::Px(1.0)),
                        position_type: PositionType::Relative,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::right(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                    BorderColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
                ))
                .with_children(|slot_parent| {
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

                    slot_parent.spawn((
                        ItemNameText,
                        Text::new(""),
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                    ));

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

            parent.spawn((
                SelectedItemName,
                Text::new(""),
                Node {
                    align_self: AlignSelf::Center,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
            ));
        })
        .id();

    selected_ui.entity = Some(ui_entity);
}

fn update_selected_item_ui(
    q_players: Query<(Entity, &Inventory), With<InteractionPlayer>>,
    q_items: Query<&Item>,
    item_registry: Res<ItemRegistry>,
    selected_ui: Res<SelectedItemUi>,
    q_ui_nodes: Query<&Children, With<SelectedItemUiRoot>>,
    mut q_images: Query<&mut ImageNode>,
    mut q_item_name_text: Query<
        &mut Text,
        (
            With<ItemNameText>,
            Without<SelectedItemName>,
            Without<QuantityText>,
        ),
    >,
    mut q_quantity_text: Query<
        &mut Text,
        (
            With<QuantityText>,
            Without<ItemNameText>,
            Without<SelectedItemName>,
        ),
    >,
    mut q_selected_name: Query<
        &mut Text,
        (
            With<SelectedItemName>,
            Without<ItemNameText>,
            Without<QuantityText>,
        ),
    >,
) {
    let Some(ui_entity) = selected_ui.entity else {
        return;
    };

    for (_player_entity, inventory) in q_players.iter() {
        if let Ok(children) = q_ui_nodes.get(ui_entity) {
            let item_entity = inventory
                .selected_index
                .and_then(|idx| inventory.items.get(idx));

            for child in children.iter() {
                if let Ok(mut image_node) = q_images.get_mut(child) {
                    if let Some(&item_entity) = item_entity {
                        if let Ok(item) = q_items.get(item_entity) {
                            if let Some(icon_handle) =
                                item_registry.icons.get(&item.id)
                            {
                                image_node.image =
                                    icon_handle.clone();
                            } else {
                                image_node.image = Handle::default();
                            }
                        }
                    } else {
                        image_node.image = Handle::default();
                    }
                }

                if let Ok(mut text) = q_item_name_text.get_mut(child)
                {
                    if let Some(&item_entity) = item_entity {
                        if let Ok(item) = q_items.get(item_entity) {
                            let item_name = item_registry
                                .by_id
                                .get(&item.id)
                                .map(|m| m.name.as_str())
                                .unwrap_or("Unknown");
                            if item_registry
                                .icons
                                .get(&item.id)
                                .is_none()
                            {
                                text.0 = item_name.to_string();
                            } else {
                                text.0 = String::new();
                            }
                        }
                    } else {
                        text.0 = String::new();
                    }
                }

                if let Ok(mut text) = q_quantity_text.get_mut(child) {
                    if let Some(&item_entity) = item_entity {
                        if let Ok(item) = q_items.get(item_entity) {
                            if item.quantity > 1 {
                                text.0 = item.quantity.to_string();
                            } else {
                                text.0 = String::new();
                            }
                        }
                    } else {
                        text.0 = String::new();
                    }
                }

                if let Ok(mut text) = q_selected_name.get_mut(child) {
                    if let Some(&item_entity) = item_entity {
                        if let Ok(item) = q_items.get(item_entity) {
                            let item_name = item_registry
                                .by_id
                                .get(&item.id)
                                .map(|m| m.name.as_str())
                                .unwrap_or("Unknown");
                            text.0 = item_name.to_string();
                        }
                    } else {
                        text.0 = "None".to_string();
                    }
                }
            }
        }
    }
}

#[derive(Resource, Default)]
struct InventoryUiState {
    // Track which player's inventory is currently open
    open_for_player: Option<Entity>,
    ui_entity: Option<Entity>,
}

#[derive(Resource, Default)]
struct SelectedItemUi {
    entity: Option<Entity>,
}

#[derive(Component)]
struct InventoryUiRoot;

#[derive(Component)]
struct SelectedItemUiRoot;

#[derive(Component)]
struct SelectedItemSlot;

#[derive(Component)]
struct InventorySlot {
    slot_index: usize,
}

#[derive(Component)]
struct ItemNameText;

#[derive(Component)]
struct QuantityText;

#[derive(Component)]
struct SelectedItemName;

#[derive(Component)]
pub struct LocalPlayer;
