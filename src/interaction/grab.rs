use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::reflect::Reflect;

use super::{
    InteractionPlayer, MarkedItem, Occupied, detect_interactables,
};

/// Plugin that sets up grabbing logic for interactable items.
pub(super) struct GrabPlugin;

impl Plugin for GrabPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                grab_input_system.after(detect_interactables),
                update_snapping,
            ),
        )
        .add_observer(handle_grab)
        .add_observer(handle_release);

        app.register_type::<Grabbable>().register_type::<Occupied>();
    }
}

/// Reads the E key press and the current MarkedItem to send grab or release events without PlayerAction.
// TODO: Use PlayerAction instead of KeyCode
fn grab_input_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    q_players: Query<
        (Entity, &MarkedItem, Option<&GrabState>),
        With<InteractionPlayer>,
    >,
    q_grabbable: Query<&Grabbable>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        // Handle input for each player separately
        for (player_entity, marked, grab_state) in q_players.iter() {
            let currently_holding =
                grab_state.is_some_and(|gs| gs.held.is_some());

            if currently_holding {
                commands.trigger_targets(
                    ReleaseEvent {
                        player: player_entity,
                    },
                    player_entity,
                );
            } else if let Some(target) = marked.0 {
                if q_grabbable.get(target).is_ok() {
                    commands.trigger_targets(
                        GrabEvent {
                            target,
                            player: player_entity,
                        },
                        player_entity,
                    );
                }
            }
        }
    }
}

/// Attaches the grabbed entity to the player and marks the player occupied.
fn handle_grab(
    trigger: Trigger<GrabEvent>,
    mut commands: Commands,
    q_grab_state: Query<&GrabState>,
) {
    let grab_event = trigger.event();
    let player_entity = grab_event.player;
    let target_entity = grab_event.target;

    // Check if this player is already holding something
    let already_holding = q_grab_state
        .get(player_entity)
        .is_ok_and(|grab_state| grab_state.held.is_some());

    if !already_holding {
        commands
            .entity(player_entity)
            .add_child(target_entity)
            .insert(Occupied)
            .insert(GrabState {
                held: Some(target_entity),
            });

        // Disable physics on the grabbed item
        commands.entity(target_entity).insert(RigidBodyDisabled);
    }
}

/// Detaches the held entity from the specific player and places it in front of them
fn handle_release(
    trigger: Trigger<ReleaseEvent>,
    mut commands: Commands,
    q_player_tf: Query<&GlobalTransform, With<InteractionPlayer>>,
    q_grab_state: Query<&GrabState>,
    mut q_tf: Query<&mut Transform>,
) {
    const RELEASE_DISTANCE: f32 = 2.0;

    let player_entity = trigger.event().player;

    // Get the player's current grab state
    if let Ok(grab_state) = q_grab_state.get(player_entity) {
        if let Some(held_entity) = grab_state.held {
            // Remove child relationship
            commands
                .entity(player_entity)
                .remove_children(&[held_entity]);

            // Clear player state
            commands
                .entity(player_entity)
                .remove::<Occupied>()
                .remove::<GrabState>();

            // Re-enable physics on the released item
            commands
                .entity(held_entity)
                .remove::<RigidBodyDisabled>();

            // Position the released item in front of the player
            if let (Ok(player_tf), Ok(mut item_tf)) = (
                q_player_tf.get(player_entity),
                q_tf.get_mut(held_entity),
            ) {
                let forward = player_tf.forward();
                item_tf.translation = player_tf.translation()
                    + forward * RELEASE_DISTANCE;
                item_tf.rotation = player_tf.rotation();
            }
        }
    }
}

/// Ensure the held entity stays snapped on top of the player.
fn update_snapping(
    q_players: Query<(Entity, &GrabState), With<InteractionPlayer>>,
    mut q_tf: Query<&mut Transform>,
) {
    const HEIGHT_OFFSET: f32 = 1.5;

    for (_player_entity, grab_state) in q_players.iter() {
        if let Some(held_entity) = grab_state.held {
            if let Ok(mut item_tf) = q_tf.get_mut(held_entity) {
                // Place item at player's head height
                item_tf.translation = Vec3::Y * HEIGHT_OFFSET;
                item_tf.rotation = Quat::IDENTITY;
            }
        }
    }
}

/// Marks an entity as grabbable.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Grabbable;

/// Tracks the currently held entity if any.
#[derive(Component, Default)]
pub struct GrabState {
    pub held: Option<Entity>,
}

/// Event to request grabbing a specified entity by a specific player
#[derive(Event)]
pub struct GrabEvent {
    pub target: Entity,
    pub player: Entity,
}

/// Event to request releasing the currently held entity from a specific player
#[derive(Event)]
pub struct ReleaseEvent {
    pub player: Entity,
}
