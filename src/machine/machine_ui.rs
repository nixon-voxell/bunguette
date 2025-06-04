use bevy::color::palettes::tailwind::*;
use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;

use crate::camera_controller::split_screen::{
    CameraType, QueryCameras,
};
use crate::interaction::MarkerPlayers;
use crate::player::PlayerType;
use crate::ui::world_space::WorldUi;

use super::recipe::{RecipeMeta, RecipeRegistry};
use super::{Machine, OperationTimer};

pub(super) struct MachineUiPlugin;

impl Plugin for MachineUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(setup_machine_ui).add_systems(
            Update,
            (machine_ui_visibility, machine_ui_content),
        );

        app.register_type::<Machine>();
    }
}

/// Setup world space popup UI for machines
fn setup_machine_ui(
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

    fn ui_bundle(machine_entity: Entity) -> impl Bundle {
        (
            WorldUi::new(machine_entity)
                .with_world_offset(Vec3::Y * 0.2),
            MachineUiOf(machine_entity),
            Node {
                padding: UiRect::all(Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            BorderRadius::all(Val::Px(6.0)),
            BackgroundColor(ZINC_900.with_alpha(0.7).into()),
            BoxShadow::new(
                ZINC_900.into(),
                Val::Px(4.0),
                Val::Px(4.0),
                Val::Px(14.0),
                Val::Px(12.0),
            ),
        )
    }

    // Create UI for both cameras
    commands
        .spawn((ui_bundle(machine_entity), UiTargetCamera(camera_a)));

    commands
        .spawn((ui_bundle(machine_entity), UiTargetCamera(camera_b)));
}

/// Set visibility of machine ui based on whether it is marked
/// by the player.
fn machine_ui_visibility(
    q_machines: Query<
        (Option<&MarkerPlayers>, &MachineUis),
        With<Machine>,
    >,
    q_target_cameras: Query<&UiTargetCamera>,
    q_camera_types: Query<&CameraType>,
    q_player_types: Query<&PlayerType>,
    mut q_nodes: Query<&mut Node>,
) -> Result {
    for (players, uis) in q_machines.iter() {
        let mut marked_by_players = vec![];

        if let Some(players) = players {
            for player in players.iter() {
                marked_by_players.push(*q_player_types.get(player)?);
            }
        }

        for ui in uis.iter() {
            let camera_type = q_target_cameras
                .get(ui)
                .and_then(|t| q_camera_types.get(t.entity()))?;

            let player_type = match camera_type {
                CameraType::A => PlayerType::A,
                CameraType::B => PlayerType::B,
                CameraType::Full => unreachable!(),
            };

            // Set node visibility based on who marked the machine.
            let mut node = q_nodes.get_mut(ui)?;
            if marked_by_players.contains(&player_type) {
                node.display = Display::DEFAULT;
            } else {
                node.display = Display::None;
            }
        }
    }

    Ok(())
}

/// System to update machine popup UI content based on machine state
fn machine_ui_content(
    mut commands: Commands,
    q_machines: Query<(&Machine, Option<&OperationTimer>, Entity)>,
    q_machine_uis: Query<(Entity, &MachineUiOf)>,
    recipe_registry: RecipeRegistry,
) {
    // Update each content marker with its specific machine's data
    for (root_id, ui_of) in q_machine_uis.iter() {
        // Find the machine that owns this content marker
        let Ok((machine, operation_timer, machine_entity)) =
            q_machines.get(ui_of.entity())
        else {
            continue;
        };

        // Update this specific machine's content
        let commands: &mut Commands = &mut commands;
        let recipe_registry: &RecipeRegistry = &recipe_registry;
        // Clear existing children
        commands.entity(root_id).despawn_related::<Children>();

        // Handle empty recipe ID
        if machine.recipe_id.is_empty() {
            error!("No recipe set for machine {machine_entity}!");
            continue;
        }

        let Some(recipe) = machine.get_recipe(recipe_registry) else {
            error!(
                "Recipe: {} does not exists for {machine_entity}!",
                machine.recipe_id
            );
            continue;
        };

        if let Some(operation_timer) = operation_timer {
            operating_machine_ui(
                commands,
                root_id,
                machine,
                recipe,
                &operation_timer.0,
            );
        } else {
            freed_machine_ui(commands, root_id, machine, recipe);
        };
    }
}

fn freed_machine_ui(
    commands: &mut Commands,
    content_entity: Entity,
    machine: &Machine,
    recipe: &RecipeMeta,
) {
    // Recipe name.
    let mut children = vec![
        commands
            .spawn((
                Text::new(
                    machine
                        .recipe_id
                        .replace('_', " ")
                        .to_uppercase(),
                ),
                TextLayout::new_with_justify(JustifyText::Center),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(CYAN_300.into()),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ))
            .id(),
    ];

    // Ingredients.
    for ingredient in recipe.ingredients.iter() {
        children.push(
            commands
                .spawn((
                    Text::new(format!(
                        "{} x{}",
                        ingredient.item_id.replace('_', " "),
                        ingredient.quantity
                    )),
                    TextLayout::new_with_justify(JustifyText::Center),
                    TextFont {
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(SLATE_200.into()),
                    Node {
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },
                ))
                .id(),
        );
    }

    children.extend([
        // Separator line.
        commands
            .spawn((
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(SLATE_600.into()),
            ))
            .id(),
        // Output.
        commands
            .spawn((
                Text::new(format!(
                    "{} x{}",
                    recipe.output_id.replace('_', " "),
                    recipe.output_quantity
                )),
                TextLayout::new_with_justify(JustifyText::Center),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(BLUE_300.into()),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ))
            .id(),
        // Cooking time.
        commands
            .spawn((
                Text::new(format!(
                    "Cooking Time: {:.1}s",
                    recipe.cooking_duration
                )),
                TextLayout::new_with_justify(JustifyText::Center),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(GRAY_400.into()),
            ))
            .id(),
    ]);

    commands.entity(content_entity).add_children(&children);
}

fn operating_machine_ui(
    commands: &mut Commands,
    content_entity: Entity,
    machine: &Machine,
    recipe: &RecipeMeta,
    timer: &Timer,
) {
    let remaining_time = timer.remaining_secs();
    let progress =
        timer.elapsed_secs() / timer.duration().as_secs_f32();

    let children = [
        // Recipe name.
        commands
            .spawn((
                Text::new(
                    machine
                        .recipe_id
                        .replace('_', " ")
                        .to_uppercase(),
                ),
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
            ))
            .id(),
        // Status.
        commands
            .spawn((
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
            ))
            .id(),
        // Time remaining.
        commands
            .spawn((
                Text::new(format!(
                    "{:.1}s remaining",
                    remaining_time
                )),
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
            ))
            .id(),
        // Progress bar container.
        commands
            .spawn((
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
            ))
            .id(),
        // Output preview.
        commands
            .spawn((
                Text::new(format!(
                    "Producing: {} x{}",
                    recipe.output_id.replace('_', " "),
                    recipe.output_quantity
                )),
                TextLayout::new_with_justify(JustifyText::Center),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(BLUE_200.into()),
            ))
            .id(),
    ];

    commands.entity(content_entity).add_children(&children);
}

#[derive(Component, Deref, Debug)]
#[relationship_target(relationship = MachineUiOf)]
pub struct MachineUis(Vec<Entity>);

/// Relation target for [`MachineUis`], relating the Ui for the [`Machine`].
#[derive(Component, Deref, Debug)]
#[relationship(relationship_target = MachineUis)]
pub struct MachineUiOf(Entity);
