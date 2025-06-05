use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use std::collections::HashMap;

use crate::camera_controller::UI_RENDER_LAYER;
use crate::interaction::InteractionPlayer;
use crate::player::PlayerType;

use crate::inventory::Inventory;
use crate::inventory::item::ItemRegistry;

pub struct InventoryUiPlugin;

impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_ingredient_display, update_tower_display),
        )
        .init_resource::<IngredientUI>()
        .init_resource::<TowerUI>();
    }
}

#[derive(Resource, Default)]
struct IngredientUI {
    // player_entity -> Vec<ingredient_ui_entities>
    ingredient_entities: HashMap<Entity, Vec<Entity>>,
}

#[derive(Resource, Default)]
struct TowerUI {
    // player_entity -> Vec<tower_ui_entities>
    tower_entities: HashMap<Entity, Vec<Entity>>,
}

/// System that displays towers on the bottom right
fn update_tower_display(
    mut commands: Commands,
    q_players: Query<
        (Entity, &Inventory, &PlayerType),
        With<InteractionPlayer>,
    >,
    item_registry: ItemRegistry,
    mut tower_ui: ResMut<TowerUI>,
) {
    for (player_entity, inventory, player_type) in q_players.iter() {
        // Clear existing UI elements for this player
        if let Some(existing_entities) =
            tower_ui.tower_entities.get(&player_entity)
        {
            for &entity in existing_entities {
                commands.entity(entity).despawn();
            }
        }

        // Collect towers to display
        let towers: Vec<_> = inventory
            .towers()
            .iter()
            .filter(|(_, count)| **count > 0)
            .collect();

        if towers.is_empty() {
            tower_ui.tower_entities.insert(player_entity, Vec::new());
            continue;
        }

        // Calculate position based on player type (bottom right)
        let base_x = match player_type {
            PlayerType::A => 800.0, // Right side for Player A
            PlayerType::B => 1760.0, // Right side for Player B
        };
        let base_y = 20.0; // Bottom

        let mut new_entities = Vec::new();

        // Create UI elements for each tower
        for (i, (tower_id, count)) in towers.iter().enumerate() {
            // Check if this tower is selected
            let is_selected =
                inventory.selected_tower.as_ref() == Some(tower_id);

            // Space items 80px apart
            let x_offset = (i as f32) * 80.0;

            //  Determine colors and border based on selection state
            let (background_color, border_color, border_width) =
                if is_selected {
                    (
                        Color::srgba(1.0, 0.6, 0.3, 0.9),
                        Color::srgba(1.0, 1.0, 0.0, 1.0),
                        4.0,
                    )
                } else {
                    (
                        Color::srgba(0.8, 0.4, 0.2, 0.8),
                        Color::srgba(1.0, 1.0, 1.0, 1.0),
                        2.0,
                    )
                };

            // Create the tower item
            let tower_entity = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(base_x + x_offset),
                        bottom: Val::Px(base_y),
                        width: Val::Px(70.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(border_width)),
                        ..default()
                    },
                    BackgroundColor(background_color),
                    BorderColor(border_color),
                    FocusPolicy::Block,
                    UI_RENDER_LAYER,
                ))
                .with_children(|parent| {
                    // Try to show icon if available
                    if let Some(item_meta_asset) = item_registry.get()
                    {
                        if let Some(meta) =
                            item_meta_asset.get(*tower_id)
                        {
                            parent.spawn((
                                ImageNode::new(meta.icon.clone()),
                                Node {
                                    width: Val::Px(40.0),
                                    height: Val::Px(40.0),
                                    margin: UiRect::bottom(Val::Px(
                                        4.0,
                                    )),
                                    ..default()
                                },
                            ));
                        } else {
                            // Show tower name if no icon found
                            parent.spawn((
                                Text::new(
                                    tower_id
                                        .chars()
                                        .take(4)
                                        .collect::<String>(),
                                ),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Node {
                                    margin: UiRect::bottom(Val::Px(
                                        4.0,
                                    )),
                                    ..default()
                                },
                            ));
                        }
                    } else {
                        // No registry loaded, show tower ID
                        parent.spawn((
                            Text::new(
                                tower_id
                                    .chars()
                                    .take(4)
                                    .collect::<String>(),
                            ),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                margin: UiRect::bottom(Val::Px(4.0)),
                                ..default()
                            },
                        ));
                    }

                    // Show quantity
                    let text_color = if is_selected {
                        // Yellow text for selected
                        Color::srgba(1.0, 1.0, 0.0, 1.0)
                        // White text for normal
                    } else {
                        Color::WHITE
                    };

                    parent.spawn((
                        Text::new(count.to_string()),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(text_color),
                    ));
                })
                .id();

            new_entities.push(tower_entity);
        }

        // Store the new entities
        tower_ui.tower_entities.insert(player_entity, new_entities);
    }
}

/// Simple system that directly creates/updates ingredient UI elements
fn update_ingredient_display(
    mut commands: Commands,
    q_players: Query<
        (Entity, &Inventory, &PlayerType),
        With<InteractionPlayer>,
    >,
    item_registry: ItemRegistry,
    mut ingredient_ui: ResMut<IngredientUI>,
) {
    for (player_entity, inventory, player_type) in q_players.iter() {
        // Clear existing UI elements for this player
        if let Some(existing_entities) =
            ingredient_ui.ingredient_entities.get(&player_entity)
        {
            for &entity in existing_entities {
                commands.entity(entity).despawn();
            }
        }

        // Collect ingredients to display
        let ingredients: Vec<_> = inventory
            .ingredients()
            .iter()
            .filter(|(_, count)| **count > 0)
            .collect();

        if ingredients.is_empty() {
            ingredient_ui
                .ingredient_entities
                .insert(player_entity, Vec::new());
            continue;
        }

        // Calculate position based on player type
        let base_x = match player_type {
            PlayerType::A => 20.0,   // Left side
            PlayerType::B => 1000.0, // Right side
        };
        let base_y = 20.0; // Bottom

        let mut new_entities = Vec::new();

        // Create UI elements for each ingredient
        for (i, (ingredient_id, count)) in
            ingredients.iter().enumerate()
        {
            let x_offset = (i as f32) * 80.0; // Space items 80px apart

            // Create the ingredient item
            let ingredient_entity = commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(base_x + x_offset),
                        bottom: Val::Px(base_y),
                        width: Val::Px(70.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.4, 0.8, 0.8)), // Blue background
                    BorderColor(Color::srgba(1.0, 1.0, 1.0, 1.0)), // White border
                    FocusPolicy::Block,
                    UI_RENDER_LAYER,
                ))
                .with_children(|parent| {
                    // Try to show icon if available
                    if let Some(item_meta_asset) = item_registry.get()
                    {
                        if let Some(meta) =
                            item_meta_asset.get(*ingredient_id)
                        {
                            parent.spawn((
                                ImageNode::new(meta.icon.clone()),
                                Node {
                                    width: Val::Px(40.0),
                                    height: Val::Px(40.0),
                                    margin: UiRect::bottom(Val::Px(
                                        4.0,
                                    )),
                                    ..default()
                                },
                            ));
                        } else {
                            // Show ingredient name if no icon found
                            parent.spawn((
                                Text::new(
                                    ingredient_id
                                        .chars()
                                        .take(4)
                                        .collect::<String>(),
                                ),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                Node {
                                    margin: UiRect::bottom(Val::Px(
                                        4.0,
                                    )),
                                    ..default()
                                },
                            ));
                        }
                    } else {
                        // No registry loaded, show ingredient ID
                        parent.spawn((
                            Text::new(
                                ingredient_id
                                    .chars()
                                    .take(4)
                                    .collect::<String>(),
                            ),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            Node {
                                margin: UiRect::bottom(Val::Px(4.0)),
                                ..default()
                            },
                        ));
                    }

                    // Show quantity
                    parent.spawn((
                        Text::new(count.to_string()),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                })
                .id();

            new_entities.push(ingredient_entity);
        }

        // Store the new entities
        ingredient_ui
            .ingredient_entities
            .insert(player_entity, new_entities);
    }
}
