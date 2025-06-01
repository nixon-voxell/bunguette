use bevy::color::palettes::tailwind::*;
use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;

use crate::camera_controller::split_screen::{
    CameraType, QueryCameras,
};
use crate::inventory::item::ItemRegistry;
use crate::ui::world_space::WorldUi;

use super::recipe::RecipeRegistry;
use super::{Machine, MachineState};
use crate::inventory::item::ItemMetaAsset;

pub(super) struct MachineUiPlugin;

impl Plugin for MachineUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_machine_popup_ui)
            .add_systems(Update, update_machine_popup_ui);

        app.register_type::<Machine>()
            .register_type::<MachinePopupUi>();
    }
}

/// Setup world space popup UI for machines
fn setup_machine_popup_ui(
    trigger: Trigger<OnAdd, Machine>,
    mut commands: Commands,
    q_cameras: QueryCameras<Entity>,
) {
    let machine_entity = trigger.target();

    let Ok(camera_a) = q_cameras.get(CameraType::A) else {
        warn!("Camera A not found when setting up machine UI");
        return;
    };
    let Ok(camera_b) = q_cameras.get(CameraType::B) else {
        warn!("Camera B not found when setting up machine UI");
        return;
    };

    let ui_bundle = move |height: f32| {
        (
            WorldUi::new(machine_entity)
                .with_world_offset(Vec3::Y * height),
            Node {
                padding: UiRect::all(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BorderRadius::all(Val::Px(12.0)),
            BackgroundColor(SLATE_800.with_alpha(0.95).into()),
            BoxShadow::new(
                Color::BLACK.with_alpha(0.2),
                Val::Px(0.0),
                Val::Px(4.0),
                Val::Px(16.0),
                Val::Px(0.0),
            ),
            Children::spawn(SpawnWith(
                move |parent: &mut ChildSpawner| {
                    // Title
                    parent.spawn((
                        Text::new("Machine"),
                        TextLayout::new_with_justify(
                            JustifyText::Center,
                        ),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(SLATE_100.into()),
                        Node {
                            margin: UiRect::bottom(Val::Px(16.0)),
                            ..default()
                        },
                    ));

                    parent.spawn((
                        MachineContentMarker { machine_entity },
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            min_width: Val::Px(220.0),
                            ..default()
                        },
                        Children::spawn(SpawnWith(
                            move |content_parent: &mut ChildSpawner| {
                                content_parent.spawn((
                                    Text::new("No Recipe Set"),
                                    TextLayout::new_with_justify(JustifyText::Center),
                                    TextFont {
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(GRAY_400.into()),
                                ));
                            },
                        )),
                    ));
                },
            )),
        )
    };

    // Create UI for both cameras
    let ui_entity_a = commands
        .spawn((ui_bundle(2.0), UiTargetCamera(camera_a)))
        .id();

    let _ui_entity_b = commands
        .spawn((ui_bundle(2.0), UiTargetCamera(camera_b)))
        .id();

    // Store UI entity reference on the machine
    commands.entity(machine_entity).insert(MachinePopupUi {
        ui_entity: ui_entity_a,
    });
}

/// System to update machine popup UI content based on machine state
fn update_machine_popup_ui(
    mut commands: Commands,
    time: Res<Time>,
    mut q_machines: Query<(&mut Machine, &MachinePopupUi)>,
    q_content_markers: Query<(Entity, &MachineContentMarker)>,
    item_registry: ItemRegistry,
    recipe_registry: RecipeRegistry,
) {
    let Some(item_meta_asset) = item_registry.get() else {
        return;
    };

    let Some(_recipes) = recipe_registry.get() else {
        return;
    };

    // Update cooking timers for all machines
    for (mut machine, _popup_ui) in q_machines.iter_mut() {
        if matches!(machine.state, MachineState::Occupied) {
            machine.elapsed_time += time.delta_secs();
        }
    }

    // Update each content marker with its specific machine's data
    for (content_entity, content_marker) in q_content_markers.iter() {
        // Find the machine that owns this content marker
        let Ok((machine, _)) =
            q_machines.get(content_marker.machine_entity)
        else {
            continue;
        };

        // Update this specific machine's content
        update_machine_content(
            &mut commands,
            content_entity,
            &machine,
            item_meta_asset,
            &recipe_registry,
        );
    }
}

fn update_machine_content(
    commands: &mut Commands,
    content_entity: Entity,
    machine: &Machine,
    _item_meta_asset: &ItemMetaAsset,
    recipe_registry: &RecipeRegistry,
) {
    // Clear existing children
    commands
        .entity(content_entity)
        .despawn_related::<Children>();

    // Handle empty recipe ID
    if machine.recipe_id.is_empty() {
        commands.entity(content_entity).insert(Children::spawn(
            SpawnWith(move |parent: &mut ChildSpawner| {
                parent.spawn((
                    Text::new("No Recipe Set"),
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(GRAY_400.into()),
                ));
            }),
        ));
        return;
    }

    let Some(recipe) = machine.get_recipe(recipe_registry) else {
        let recipe_id = machine.recipe_id.clone();
        commands.entity(content_entity).insert(Children::spawn(
            SpawnWith(move |parent: &mut ChildSpawner| {
                parent.spawn((
                    Text::new(format!(
                        "Recipe '{}' not found",
                        recipe_id
                    )),
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(RED_400.into()),
                ));
            }),
        ));
        return;
    };

    // Extract recipe details
    let recipe_id = machine.recipe_id.clone();
    let machine_state = machine.state.clone();
    let remaining_time = machine.remaining_time(recipe_registry);
    let progress = machine.cooking_progress(recipe_registry);
    let ingredients = recipe.ingredients.clone();
    let output_id = recipe.output_id.clone();
    let output_quantity = recipe.output_quantity;
    let cooking_duration = recipe.cooking_duration;

    match machine_state {
        MachineState::Ready => {
            commands.entity(content_entity).insert(Children::spawn(
                SpawnWith(move |parent: &mut ChildSpawner| {
                    // Recipe name
                    parent.spawn((
                        Text::new(
                            recipe_id
                                .replace('_', " ")
                                .to_uppercase(),
                        ),
                        TextLayout::new_with_justify(
                            JustifyText::Center,
                        ),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(CYAN_300.into()),
                        Node {
                            margin: UiRect::bottom(Val::Px(12.0)),
                            ..default()
                        },
                    ));

                    // Ingredients
                    for ingredient in &ingredients {
                        let color = SLATE_200;

                        parent.spawn((
                            Text::new(format!(
                                "{} x{}",
                                ingredient.item_id.replace('_', " "),
                                ingredient.quantity
                            )),
                            TextLayout::new_with_justify(
                                JustifyText::Center,
                            ),
                            TextFont {
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(color.into()),
                            Node {
                                margin: UiRect::bottom(Val::Px(4.0)),
                                ..default()
                            },
                        ));
                    }

                    // Separator line
                    parent.spawn((
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(SLATE_600.into()),
                    ));

                    // Output
                    parent.spawn((
                        Text::new(format!(
                            "{} x{}",
                            output_id.replace('_', " "),
                            output_quantity
                        )),
                        TextLayout::new_with_justify(
                            JustifyText::Center,
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(BLUE_300.into()),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    // Cooking time
                    parent.spawn((
                        Text::new(format!(
                            "Cooking Time: {:.1}s",
                            cooking_duration
                        )),
                        TextLayout::new_with_justify(
                            JustifyText::Center,
                        ),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(GRAY_400.into()),
                    ));
                }),
            ));
        }
        MachineState::Occupied => {
            commands.entity(content_entity).insert(Children::spawn(SpawnWith(
                move |parent: &mut ChildSpawner| {
                    // Recipe name
                    parent.spawn((
                        Text::new(recipe_id.replace('_', " ").to_uppercase()),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(ORANGE_300.into()),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    // Status
                    parent.spawn((
                        Text::new("Cooking..."),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(YELLOW_200.into()),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    // Time remaining
                    parent.spawn((
                        Text::new(format!("{:.1}s remaining", remaining_time)),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(SLATE_200.into()),
                        Node {
                            margin: UiRect::bottom(Val::Px(12.0)),
                            ..default()
                        },
                    ));

                    // Progress bar container
                    parent.spawn((
                        Node {
                            width: Val::Px(140.0),
                            height: Val::Px(8.0),
                            margin: UiRect::bottom(Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(GRAY_700.into()),
                        BorderRadius::all(Val::Px(4.0)),
                        Children::spawn(SpawnWith(
                            move |progress_parent: &mut ChildSpawner| {
                                // Progress bar fill
                                progress_parent.spawn((
                                    Node {
                                        width: Val::Percent(progress * 100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(ORANGE_400.into()),
                                    BorderRadius::all(Val::Px(4.0)),
                                ));
                            },
                        )),
                    ));

                    // Output preview
                    parent.spawn((
                        Text::new(format!(
                            "Producing: {} x{}",
                            output_id.replace('_', " "),
                            output_quantity
                        )),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(BLUE_200.into()),
                    ));
                },
            )));
        }
    }
}

/// Component linking a machine to its popup UI entity
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct MachinePopupUi {
    pub ui_entity: Entity,
}

/// Marker component for the content section of machine popup UI
#[derive(Component)]
struct MachineContentMarker {
    machine_entity: Entity,
}
