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
        app.init_resource::<GrabState>()
            .add_systems(
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
    q_marked: Query<(Entity, &MarkedItem), With<InteractionPlayer>>,
    q_grabbable: Query<&Grabbable>,
    grab_state: Res<GrabState>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        if grab_state.held.is_some() {
            commands.trigger(ReleaseEvent);
        } else if let Ok((_, marked)) = q_marked.single() {
            if let Some(target) = marked.0 {
                if q_grabbable.get(target).is_ok() {
                    // Send grab event
                    commands.trigger(GrabEvent(target));
                }
            }
        }
    }
}

/// Attaches the grabbed entity to the player and marks the player occupied.
fn handle_grab(
    trigger: Trigger<GrabEvent>,
    mut commands: Commands,
    mut grab_state: ResMut<GrabState>,
    player_q: Query<Entity, With<InteractionPlayer>>,
) {
    let grab_event = trigger.event();
    let entity = grab_event.0;

    if grab_state.held.is_none() {
        if let Ok(player) = player_q.single() {
            commands
                .entity(player)
                .add_child(entity)
                .insert(Occupied)
                .insert(RigidBodyDisabled);
            grab_state.held = Some(entity);
        }
    }
}

/// Detaches the held entity and places it in front of the player.
fn handle_release(
    _trigger: Trigger<ReleaseEvent>,
    mut commands: Commands,
    mut grab_state: ResMut<GrabState>,
    q_player: Query<Entity, With<InteractionPlayer>>,
    q_player_tf: Query<&GlobalTransform, With<InteractionPlayer>>,
    mut q_tf: Query<&mut Transform>,
) {
    // Release distance from player
    const RELEASE_DISTANCE: f32 = 2.0;

    if let Some(entity) = grab_state.held.take() {
        if let Ok(player) = q_player.single() {
            commands.entity(player).remove_children(&[entity]);
            // Clear occupied tag
            commands.entity(player).remove::<Occupied>();
            if let (Ok(player_tf), Ok(mut item_tf)) =
                (q_player_tf.single(), q_tf.get_mut(entity))
            {
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
    grab_state: Res<GrabState>,
    mut q_tf: Query<&mut Transform>,
) {
    if let Some(entity) = grab_state.held {
        if let Ok(mut item_tf) = q_tf.get_mut(entity) {
            const HEIGHT_OFFSET: f32 = 1.5;
            // Place item at player's head height
            item_tf.translation = Vec3::Y * HEIGHT_OFFSET;

            // Rotate the held item to match the player's rotation
            item_tf.rotation = Quat::IDENTITY;
        }
    }
}

/// Marks an entity as grabbable.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Grabbable;

/// Tracks the currently held entity if any.
#[derive(Resource, Default)]
pub struct GrabState {
    pub held: Option<Entity>,
}

/// Event to request grabbing a specified entity.
#[derive(Event)]
pub struct GrabEvent(pub Entity);

/// Event to request releasing the currently held entity.
#[derive(Event)]
pub struct ReleaseEvent;
