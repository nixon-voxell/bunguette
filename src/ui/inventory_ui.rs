use bevy::color::palettes::tailwind::*;
use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::camera_controller::UI_RENDER_LAYER;
use crate::interaction::InteractionPlayer;
use crate::player::PlayerType;

use crate::inventory::Inventory;
use crate::inventory::item::ItemRegistry;

pub struct InventoryUiPlugin;

impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, split_screen_ui).add_systems(
            Update,
            (clear_inventory_ui, spawn_inventory_ui).chain(),
        );
    }
}

fn clear_inventory_ui(
    mut commands: Commands,
    inventory_ui: Res<InventoryUi>,
) {
    [
        inventory_ui.a_towers,
        inventory_ui.a_ingredients,
        inventory_ui.b_towers,
        inventory_ui.b_ingredients,
    ]
    .iter()
    .for_each(|e| {
        commands.entity(*e).despawn_related::<Children>();
    });
}

fn spawn_inventory_ui(
    mut commands: Commands,
    q_players: Query<
        (&Inventory, &PlayerType),
        With<InteractionPlayer>,
    >,
    item_registry: ItemRegistry,
    inventory_ui: Res<InventoryUi>,
) -> Result {
    for (inventory, player_type) in q_players.iter() {
        let (tower_node, ingredient_node) = match player_type {
            PlayerType::A => {
                (inventory_ui.a_towers, inventory_ui.a_ingredients)
            }
            PlayerType::B => {
                (inventory_ui.b_towers, inventory_ui.b_ingredients)
            }
        };

        let item_bundle =
            |border_width: f32,
             bg_color: Color,
             border_color: Color,
             item_id: &str,
             item_count: u32| {
                Result::<_, String>::Ok((
                    Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(border_width)),
                        margin: UiRect::horizontal(Val::Px(20.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(bg_color.with_alpha(0.5)),
                    BorderColor(border_color.with_alpha(0.7)),
                    BorderRadius::all(Val::Px(8.0)),
                    BoxShadow::new(
                        bg_color.with_alpha(0.8),
                        Val::Px(2.0),
                        Val::Px(2.0),
                        Val::Px(4.0),
                        Val::Px(6.0),
                    ),
                    Children::spawn((
                        Spawn((
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(80.0),
                                margin: UiRect::bottom(Val::Px(4.0)),
                                padding: UiRect::all(Val::Px(4.0)),
                                ..default()
                            },
                            ImageNode::new(
                                item_registry
                                    .get_item(item_id)
                                    .ok_or(format!(
                                        "No icon for tower {item_id}"
                                    ))?
                                    .icon
                                    .clone(),
                            ),
                        )),
                        Spawn((
                            Text::new(item_count.to_string()),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(border_color),
                        )),
                    )),
                ))
            };

        for (tower_id, count) in
            inventory.towers().iter().filter(|(_, count)| **count > 0)
        {
            // Check if this tower is selected
            let is_selected =
                inventory.selected_tower.as_ref() == Some(tower_id);

            //  Determine colors and border based on selection state
            let (bg_color, border_color) = if is_selected {
                (EMERALD_800, EMERALD_500)
            } else {
                (SLATE_800, SLATE_200)
            };

            // Create the item node.
            let tower_item_node = commands
                .spawn(item_bundle(
                    2.0,
                    bg_color.into(),
                    border_color.into(),
                    tower_id,
                    *count,
                )?)
                .id();

            commands.entity(tower_node).add_child(tower_item_node);
        }

        for (ingredient_id, count) in inventory
            .ingredients()
            .iter()
            .filter(|(_, count)| **count > 0)
        {
            // Create the item node.
            let ingredient_item_node = commands
                .spawn(item_bundle(
                    2.0,
                    SLATE_800.into(),
                    SLATE_200.into(),
                    ingredient_id,
                    *count,
                )?)
                .id();

            commands
                .entity(ingredient_node)
                .add_child(ingredient_item_node);
        }
    }

    Ok(())
}

/// Create split screen ui.
fn split_screen_ui(mut commands: Commands) {
    let split_bundle =
        |tower_node: Entity, ingreient_node: Entity| {
            (
                Node {
                    // Takes half the space.
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    // Push the child node towards the bottom.
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::End,
                    ..default()
                },
                FocusPolicy::Pass,
                Pickable::IGNORE,
                Children::spawn(SpawnWith(
                    move |parent: &mut ChildSpawner| {
                        parent
                            .spawn((
                                Node {
                                    flex_direction:
                                        FlexDirection::Row,
                                    justify_content:
                                        JustifyContent::SpaceBetween,
                                    padding: UiRect::all(Val::Px(
                                        20.0,
                                    )),
                                    ..default()
                                },
                                FocusPolicy::Pass,
                                Pickable::IGNORE,
                            ))
                            .add_children(&[
                                tower_node,
                                ingreient_node,
                            ]);
                    },
                )),
            )
        };

    let items_bundle = (
        Node {
            flex_direction: FlexDirection::Row,
            ..default()
        },
        FocusPolicy::Pass,
        Pickable::IGNORE,
    );

    let a_towers = commands.spawn(items_bundle.clone()).id();
    let a_ingredients = commands.spawn(items_bundle.clone()).id();

    let b_towers = commands.spawn(items_bundle.clone()).id();
    let b_ingredients = commands.spawn(items_bundle).id();

    commands.spawn((
        UI_RENDER_LAYER,
        // Root node.
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        FocusPolicy::Pass,
        Pickable::IGNORE,
        Children::spawn((
            Spawn(split_bundle(a_towers, a_ingredients)),
            Spawn(split_bundle(b_towers, b_ingredients)),
        )),
    ));

    commands.insert_resource(InventoryUi {
        a_towers,
        a_ingredients,
        b_towers,
        b_ingredients,
    });
}

#[derive(Resource, Debug)]
pub struct InventoryUi {
    pub a_towers: Entity,
    pub a_ingredients: Entity,
    pub b_towers: Entity,
    pub b_ingredients: Entity,
}
