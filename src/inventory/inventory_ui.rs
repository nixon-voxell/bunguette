use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leafwing_input_manager::prelude::ActionState;
use std::collections::HashMap;

use crate::action::PlayerAction;
use crate::camera_controller::CameraType;
use crate::camera_controller::UI_RENDER_LAYER;
use crate::interaction::InteractionPlayer;
use crate::player::PlayerType;

use super::{Inventory, Item, ItemRegistry};

pub struct InventoryUiPlugin;

impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                toggle_inventory,
                update_inventory_ui
                    .run_if(resource_exists::<InventoryUiState>),
                update_selected_item_ui,
                ensure_selected_item_hud_for_players,
            ),
        )
        .init_resource::<InventoryUiState>()
        .init_resource::<SelectedItemUi>();
    }
}

/// Toggle inventory UI with Tab key - works for local player or first player found
fn toggle_inventory(
    mut commands: Commands,
    q_players: Query<
        (Entity, &ActionState<PlayerAction>, &PlayerType),
        With<InteractionPlayer>,
    >,
    q_inventories: Query<&Inventory, With<InteractionPlayer>>,
    mut ui_state: ResMut<InventoryUiState>,
    q_cameras: Query<(&Camera, &CameraType)>,
) {
    for (player_entity, action_state, player_type) in q_players.iter()
    {
        if action_state.just_pressed(&PlayerAction::ToggleInventory) {
            if let Some(ui_entity) =
                ui_state.open_for_players.remove(&player_entity)
            {
                // Only despawn the inventory UI, not the HUD
                commands.entity(ui_entity).despawn();
            } else if let Ok(inventory) =
                q_inventories.get(player_entity)
            {
                let ui_entity = spawn_inventory_ui(
                    &mut commands,
                    inventory.capacity,
                    *player_type,
                    q_cameras,
                );
                ui_state
                    .open_for_players
                    .insert(player_entity, ui_entity);
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
        (
            &InventorySlot,
            &InventorySlotOwner,
            &Children,
            &mut BackgroundColor,
        ),
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
    for (&player_entity, &ui_entity) in
        ui_state.open_for_players.iter()
    {
        if let Ok(inventory) = q_inventories.get(player_entity) {
            for (slot, owner, children, mut background_color) in
                q_slots.iter_mut()
            {
                if owner.ui_entity != ui_entity {
                    continue;
                }

                let item_entity =
                    inventory.items.get(slot.slot_index);
                let is_selected =
                    inventory.selected_index == Some(slot.slot_index);

                // Check if slot has a valid item
                let is_empty = item_entity.is_none()
                    || item_entity == Some(&Entity::PLACEHOLDER)
                    || item_entity
                        .and_then(|&e| q_items.get(e).ok())
                        .is_none();

                // Update slot background
                *background_color = if is_selected {
                    BackgroundColor(Color::srgba(0.4, 0.4, 0.8, 0.9))
                } else if !is_empty {
                    BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8))
                } else {
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8))
                };

                // Update children components
                for child in children.iter() {
                    // Update image
                    if let Ok(mut image_node) =
                        q_images.get_mut(child)
                    {
                        if !is_empty {
                            if let Some(&item_entity) = item_entity {
                                if let Ok(item) =
                                    q_items.get(item_entity)
                                {
                                    if let Some(icon_handle) =
                                        item_registry
                                            .icons
                                            .get(&item.id)
                                    {
                                        image_node.image =
                                            icon_handle.clone();
                                        image_node.color =
                                            // Make visible
                                            Color::WHITE;
                                    } else {
                                        image_node.image =
                                            Handle::default();
                                        image_node.color =
                                            // Hide when no icon
                                            Color::NONE;
                                    }
                                } else {
                                    image_node.image =
                                        Handle::default();
                                    image_node.color = Color::NONE;
                                }
                            } else {
                                image_node.image = Handle::default();
                                image_node.color = Color::NONE;
                            }
                        } else {
                            // Empty slot - hide image
                            image_node.image = Handle::default();
                            image_node.color = Color::NONE;
                        }
                    }

                    // Update item name text
                    if let Ok(mut text) =
                        q_item_name_text.get_mut(child)
                    {
                        if !is_empty {
                            if let Some(&item_entity) = item_entity {
                                if let Ok(item) =
                                    q_items.get(item_entity)
                                {
                                    let item_meta = item_registry
                                        .by_id
                                        .get(&item.id);
                                    let item_name = item_meta
                                        .map(|m| m.name.as_str())
                                        .unwrap_or("Unknown");

                                    // Show text only if no icon
                                    if item_registry
                                        .icons
                                        .get(&item.id)
                                        .is_none()
                                    {
                                        text.0 =
                                            item_name.to_string();
                                    } else {
                                        // Hide text when icon exists
                                        text.0 = String::new();
                                    }
                                } else {
                                    text.0 = String::new();
                                }
                            } else {
                                text.0 = String::new();
                            }
                        } else {
                            text.0 = String::new(); // Empty slot
                        }
                    }

                    // Update quantity text
                    if let Ok(mut text) =
                        q_quantity_text.get_mut(child)
                    {
                        if !is_empty {
                            if let Some(&item_entity) = item_entity {
                                if let Ok(item) =
                                    q_items.get(item_entity)
                                {
                                    if item.quantity > 1 {
                                        text.0 =
                                            item.quantity.to_string();
                                    } else {
                                        text.0 = String::new();
                                    }
                                } else {
                                    text.0 = String::new();
                                }
                            } else {
                                text.0 = String::new();
                            }
                        } else {
                            text.0 = String::new(); // Empty slot
                        }
                    }
                }
            }
        }
    }
}

/// Handle slot clicks - select slot for interaction
fn on_slot_click(
    click: Trigger<Pointer<Click>>,
    q_slots: Query<&InventorySlot>,
    mut q_inventories: Query<&mut Inventory, With<InteractionPlayer>>,
    ui_state: Res<InventoryUiState>,
) {
    // Find which player's UI this slot belongs to
    for (&player_entity, &_ui_entity) in
        ui_state.open_for_players.iter()
    {
        if let Ok(slot) = q_slots.get(click.target()) {
            if let Ok(mut inventory) =
                q_inventories.get_mut(player_entity)
            {
                if slot.slot_index < inventory.capacity {
                    inventory.selected_index = Some(slot.slot_index);
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

/// Handle slot hover enter
fn on_slot_hover(
    trigger: Trigger<Pointer<Over>>,
    q_slots: Query<&InventorySlot>,
    mut q_backgrounds: Query<
        &mut BackgroundColor,
        With<InventorySlot>,
    >,
    q_inventories: Query<&Inventory, With<InteractionPlayer>>,
    ui_state: Res<InventoryUiState>,
) {
    for (&player_entity, &_ui_entity) in
        ui_state.open_for_players.iter()
    {
        if let Ok(slot) = q_slots.get(trigger.target()) {
            if let Ok(mut background) =
                q_backgrounds.get_mut(trigger.target())
            {
                if let Ok(inventory) =
                    q_inventories.get(player_entity)
                {
                    let is_selected = inventory.selected_index
                        == Some(slot.slot_index);
                    let _has_item = inventory
                        .items
                        .get(slot.slot_index)
                        .is_some();
                    *background = if is_selected {
                        BackgroundColor(Color::srgba(
                            0.4, 0.4, 0.8, 0.9,
                        ))
                    } else {
                        BackgroundColor(Color::srgba(
                            0.2, 0.2, 0.2, 0.8,
                        ))
                    };
                }
            }
        }
    }
}

/// Handle slot hover exit
fn on_slot_exit(
    trigger: Trigger<Pointer<Out>>,
    q_slots: Query<&InventorySlot>,
    mut q_backgrounds: Query<
        &mut BackgroundColor,
        With<InventorySlot>,
    >,
    q_inventories: Query<&Inventory, With<InteractionPlayer>>,
    ui_state: Res<InventoryUiState>,
) {
    for (&player_entity, &_ui_entity) in
        ui_state.open_for_players.iter()
    {
        if let Ok(slot) = q_slots.get(trigger.target()) {
            if let Ok(mut background) =
                q_backgrounds.get_mut(trigger.target())
            {
                if let Ok(inventory) =
                    q_inventories.get(player_entity)
                {
                    let is_selected = inventory.selected_index
                        == Some(slot.slot_index);
                    let has_item = inventory
                        .items
                        .get(slot.slot_index)
                        .is_some();
                    *background = if is_selected {
                        BackgroundColor(Color::srgba(
                            0.4, 0.4, 0.8, 0.9,
                        ))
                    } else if has_item {
                        BackgroundColor(Color::srgba(
                            0.3, 0.3, 0.3, 0.8,
                        ))
                    } else {
                        BackgroundColor(Color::srgba(
                            0.2, 0.2, 0.2, 0.8,
                        ))
                    };
                }
            }
        }
    }
}

/// Spawn the inventory UI for a specific player
fn spawn_inventory_ui(
    commands: &mut Commands,
    capacity: usize,
    player_type: PlayerType,
    q_cameras: Query<(&Camera, &CameraType)>,
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

    // Default viewport (fallback if camera not found)
    let mut viewport_x = 0.0;
    let mut viewport_width = 1920.0;
    let viewport_height = 1080.0;

    // Find the camera for the player
    for (camera, camera_type) in q_cameras.iter() {
        if (player_type == PlayerType::A
            && *camera_type == CameraType::A)
            || (player_type == PlayerType::B
                && *camera_type == CameraType::B)
        {
            if let Some(viewport) = &camera.viewport {
                viewport_x = viewport.physical_position.x as f32;
                viewport_width = viewport.physical_size.x as f32;
            }
        }
    }

    // Center the UI in the player's viewport
    let left =
        Val::Px(viewport_x + (viewport_width - total_width) / 2.0);
    let top = Val::Px((viewport_height - total_height) / 2.0);

    let ui_entity = commands
        .spawn((
            InventoryUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left,
                right: Val::Auto,
                top,
                width: Val::Px(total_width),
                height: Val::Px(total_height),
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(PANEL_PADDING)),
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::default(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
            FocusPolicy::Block,
            // Render on UI camera's layer
            UI_RENDER_LAYER,
        ))
        .id();

    commands.entity(ui_entity).with_children(|parent| {
        // Instructions text
        parent.spawn((
            Text::new("Inventory System"),
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
        parent
            .spawn((Node {
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
            },))
            .with_children(|grid_parent| {
                for slot_index in 0..capacity {
                    grid_parent
                        .spawn((
                            InventorySlot { slot_index },
                            InventorySlotOwner { ui_entity },
                            Node {
                                width: Val::Px(SLOT_SIZE),
                                height: Val::Px(SLOT_SIZE),
                                border: UiRect::all(Val::Px(1.0)),
                                position_type: PositionType::Relative,
                                justify_content:
                                    JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(
                                0.2, 0.2, 0.2, 0.8,
                            )),
                            BorderColor(Color::srgba(
                                0.5, 0.5, 0.5, 1.0,
                            )),
                        ))
                        .observe(on_slot_click)
                        .observe(on_slot_hover)
                        .observe(on_slot_exit)
                        .with_children(|slot_parent| {
                            // Slot number label (small text in corner)
                            slot_parent.spawn((
                                Text::new(
                                    (slot_index + 1).to_string(),
                                ),
                                Node {
                                    position_type:
                                        PositionType::Absolute,
                                    top: Val::Px(2.0),
                                    left: Val::Px(2.0),
                                    ..default()
                                },
                                TextColor(Color::srgba(
                                    0.7, 0.7, 0.7, 1.0,
                                )),
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
                                Node {
                                    width: Val::Percent(90.0),
                                    height: Val::Percent(90.0),
                                    position_type:
                                        PositionType::Absolute,
                                    ..default()
                                },
                            ));

                            // Item name text (fallback when no icon)
                            slot_parent.spawn((
                                ItemNameText,
                                Text::new(""),
                                Node {
                                    position_type:
                                        PositionType::Absolute,
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
                                    position_type:
                                        PositionType::Absolute,
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
    });
    ui_entity
}

/// Spawn the selected item UI for a specific player
fn spawn_selected_item_ui_for_player(
    commands: &mut Commands,
    _player_entity: Entity,
    player_type: PlayerType,
    q_cameras: Query<(&Camera, &CameraType)>,
) -> Entity {
    const SLOT_SIZE: f32 = 48.0;
    const PANEL_PADDING: f32 = 8.0;
    // Margin from viewport edges
    const MARGIN: f32 = 10.0;
    let total_width = SLOT_SIZE + (PANEL_PADDING * 2.0) + 100.0;
    let total_height = SLOT_SIZE + (PANEL_PADDING * 2.0);

    // Default viewport (fallback if camera not found)
    let mut viewport_x = 0.0;

    // Find the camera for the player
    let mut camera_found = false;
    for (camera, camera_type) in q_cameras.iter() {
        if (player_type == PlayerType::A
            && *camera_type == CameraType::A)
            || (player_type == PlayerType::B
                && *camera_type == CameraType::B)
        {
            if let Some(viewport) = &camera.viewport {
                viewport_x = viewport.physical_position.x as f32;
                let viewport_width = viewport.physical_size.x as f32;
                camera_found = true;
                info!(
                    "Player: {:?}, Viewport x: {}, width: {}",
                    player_type, viewport_x, viewport_width
                );
            }
        }
    }
    if !camera_found {
        warn!("No camera found for player {:?}", player_type);
    }

    // Position HUD in the corner
    let (left, right) = match player_type {
        // Bottom-left
        PlayerType::A => (Val::Px(viewport_x + MARGIN), Val::Auto),
        // Bottom-right
        PlayerType::B => (Val::Auto, Val::Px(MARGIN)),
    };
    let bottom = Val::Px(MARGIN);

    commands
        .spawn((
            SelectedItemUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left,
                right,
                bottom,
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
            UI_RENDER_LAYER,
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
                    // Item icon
                    slot_parent.spawn((
                        ImageNode {
                            color: Color::NONE,
                            ..default()
                        },
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
                        TextColor(Color::srgba(
                            255.0, 165.0, 0.0, 1.0,
                        )),
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
        .id()
}

/// System to ensure each player always has their own selected item HUD
fn ensure_selected_item_hud_for_players(
    mut commands: Commands,
    q_players: Query<(Entity, &PlayerType), With<InteractionPlayer>>,
    q_cameras: Query<(&Camera, &CameraType)>,
    mut selected_ui: ResMut<SelectedItemUi>,
) {
    for (player_entity, player_type) in q_players.iter() {
        if !selected_ui.entities.contains_key(&player_entity) {
            let hud_entity = spawn_selected_item_ui_for_player(
                &mut commands,
                player_entity,
                *player_type,
                q_cameras,
            );
            selected_ui.entities.insert(player_entity, hud_entity);
        }
    }
}

/// Update the selected item HUD for each player
fn update_selected_item_ui(
    q_players: Query<
        (Entity, &Inventory, &PlayerType),
        With<InteractionPlayer>,
    >,
    q_items: Query<&Item>,
    item_registry: Res<ItemRegistry>,
    selected_ui: Res<SelectedItemUi>,
    q_ui_nodes: Query<&Children, With<SelectedItemUiRoot>>,
    q_slot_children: Query<&Children, With<SelectedItemSlot>>,
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
    for (player_entity, inventory, _player_type) in q_players.iter() {
        if let Some(ui_entity) =
            selected_ui.entities.get(&player_entity)
        {
            if let Ok(children) = q_ui_nodes.get(*ui_entity) {
                // Get selected item
                let selected_item = inventory
                    .selected_index
                    .and_then(|idx| inventory.items.get(idx))
                    .and_then(|&item_entity| {
                        if item_entity == Entity::PLACEHOLDER {
                            None
                        } else {
                            q_items
                                .get(item_entity)
                                .ok()
                                .map(|item| (item_entity, item))
                        }
                    });

                // Update all child components
                for child in children.iter() {
                    // Check if this child is a SelectedItemSlot
                    if let Ok(slot_children) =
                        q_slot_children.get(child)
                    {
                        // Traverse the slot's children to find ImageNode, ItemNameText, QuantityText
                        for slot_child in slot_children.iter() {
                            // Update image
                            if let Ok(mut image_node) =
                                q_images.get_mut(slot_child)
                            {
                                if let Some((_, item)) = selected_item
                                {
                                    if let Some(icon_handle) =
                                        item_registry
                                            .icons
                                            .get(&item.id)
                                    {
                                        image_node.image =
                                            icon_handle.clone();
                                        // Make visible
                                        image_node.color =
                                            Color::WHITE;
                                    } else {
                                        image_node.image =
                                            Handle::default();
                                        // Hide when no icon
                                        image_node.color =
                                            Color::NONE;
                                    }
                                } else {
                                    image_node.image =
                                        Handle::default();
                                    // No selection
                                    image_node.color = Color::NONE;
                                }
                            }

                            // Update item name text (fallback when no icon)
                            if let Ok(mut text) =
                                q_item_name_text.get_mut(slot_child)
                            {
                                if let Some((_, item)) = selected_item
                                {
                                    let item_name = item_registry
                                        .by_id
                                        .get(&item.id)
                                        .map(|m| m.name.as_str())
                                        .unwrap_or("Unknown");

                                    // Show text only if no icon
                                    if !item_registry
                                        .icons
                                        .contains_key(&item.id)
                                    {
                                        text.0 =
                                            item_name.to_string();
                                    } else {
                                        // Hide when icon exists
                                        text.0 = String::new();
                                    }
                                } else {
                                    // No selection
                                    text.0 = String::new();
                                }
                            }

                            // Update quantity text
                            if let Ok(mut text) =
                                q_quantity_text.get_mut(slot_child)
                            {
                                if let Some((_, item)) = selected_item
                                {
                                    if item.quantity > 1 {
                                        text.0 =
                                            item.quantity.to_string();
                                    } else {
                                        text.0 = String::new();
                                    }
                                } else {
                                    // No selection
                                    text.0 = String::new();
                                }
                            }
                        }
                    }

                    // Update selected item name
                    if let Ok(mut text) =
                        q_selected_name.get_mut(child)
                    {
                        if let Some((_, item)) = selected_item {
                            let item_name = item_registry
                                .by_id
                                .get(&item.id)
                                .map(|m| m.name.as_str())
                                .unwrap_or("Unknown");
                            text.0 = item_name.to_string();
                        } else {
                            // No selection
                            text.0 = "None".to_string();
                        }
                    }
                }
            }
        }
    }
}

#[derive(Resource, Default)]
struct InventoryUiState {
    // Map player entity to their UI entity
    open_for_players: HashMap<Entity, Entity>,
}

#[derive(Resource, Default)]
struct SelectedItemUi {
    // player_entity -> ui_entity
    entities: HashMap<Entity, Entity>,
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
struct InventorySlotOwner {
    ui_entity: Entity,
}
