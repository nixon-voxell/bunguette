use crate::interaction::{
    InteractionPlayer, MarkedItem, detect_interactables,
};
use bevy::prelude::*;
use bevy::reflect::Reflect;

/// Plugin that sets up grabbing logic for interactable items.
pub(super) struct GrabPlugin;

impl Plugin for GrabPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_test_scene)
            .init_resource::<GrabState>()
            .add_event::<GrabEvent>()
            .add_event::<ReleaseEvent>()
            .add_systems(
                Update,
                (
                    grab_input_system.after(detect_interactables),
                    handle_grab,
                    handle_release,
                    update_snapping,
                ),
            );
        app.register_type::<Grabbable>().register_type::<Occupied>();
    }
}

// -- TESTING SCENE ----------------------------------------------------------------
fn spawn_test_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("scenes/grab_test.glb"),
    )));
}

/// Reads the `E` key press and the current `MarkedItem` to send grab or release events without PlayerAction.
// TODO: Use PlayerAction instead of KeyCode
fn grab_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    marked_q: Query<(Entity, &MarkedItem), With<InteractionPlayer>>,
    grabbable_q: Query<&Grabbable>,
    mut grab_writer: EventWriter<GrabEvent>,
    mut release_writer: EventWriter<ReleaseEvent>,
    grab_state: Res<GrabState>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        if grab_state.held.is_some() {
            release_writer.write(ReleaseEvent);
        } else if let Ok((_, marked)) = marked_q.single() {
            if let Some(target) = marked.0 {
                if grabbable_q.get(target).is_ok() {
                    // Send grab event
                    grab_writer.write(GrabEvent(target));
                }
            }
        }
    }
}

/// Attaches the grabbed entity to the player and marks the player occupied.
fn handle_grab(
    mut commands: Commands,
    mut grab_state: ResMut<GrabState>,
    mut events: EventReader<GrabEvent>,
    player_q: Query<Entity, With<InteractionPlayer>>,
) {
    for GrabEvent(entity) in events.read() {
        if grab_state.held.is_some() {
            continue;
        }
        if let Ok(player) = player_q.single() {
            // Parent the grabbed item to the interactable player
            commands.entity(player).add_child(*entity);
            // Reset local transform so it's positioned relative to parent
            commands
                .entity(*entity)
                .insert(Transform::default())
                .insert(GlobalTransform::default());
            // Tag as occupied
            commands.entity(player).insert(Occupied);
            grab_state.held = Some(*entity);
        }
    }
}

/// Detaches the held entity and places it in front of the player.
fn handle_release(
    mut commands: Commands,
    mut grab_state: ResMut<GrabState>,
    mut events: EventReader<ReleaseEvent>,
    player_q: Query<Entity, With<InteractionPlayer>>,
    player_tf_q: Query<&GlobalTransform, With<InteractionPlayer>>,
) {
    // Release distance from player
    const RELEASE_DISTANCE: f32 = 2.0;
    for _ in events.read() {
        if let Some(entity) = grab_state.held.take() {
            if let Ok(player) = player_q.single() {
                commands.entity(player).remove_children(&[entity]);
                // Clear occupied tag
                commands.entity(player).remove::<Occupied>();
                // Determine release position and rotation from player transform
                if let Ok(player_tf) = player_tf_q.single() {
                    let forward = player_tf.forward();
                    let release_pos = player_tf.translation()
                        + forward * RELEASE_DISTANCE;
                    let release_rot = player_tf.rotation();
                    // Place entity at computed world transform
                    commands
                        .entity(entity)
                        .insert(Transform {
                            translation: release_pos,
                            rotation: release_rot,
                            ..Default::default()
                        })
                        .insert(GlobalTransform::default());
                }
            }
        }
    }
}

/// Ensure the held entity stays snapped on top of the player.
fn update_snapping(
    grab_state: Res<GrabState>,
    mut tf_q: Query<&mut Transform>,
) {
    if let Some(entity) = grab_state.held {
        if let Ok(mut item_tf) = tf_q.get_mut(entity) {
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

/// Tags the player as occupied when holding an item.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Occupied;

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
