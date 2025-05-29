use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use leafwing_input_manager::prelude::*;
use split_screen::{
    CameraType, QueryCameraA, QueryCameraB, QueryCameraFull,
};

use crate::action::{PlayerAction, RequireAction, TargetAction};
use crate::player::PlayerType;

pub mod split_screen;

pub const UI_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

pub(super) struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(split_screen::SplitScreenPlugin);

        app.add_systems(
            PostUpdate,
            snap_camera.after(TransformSystem::TransformPropagate),
        )
        .add_systems(Update, orbit_camera)
        .add_observer(setup_directional_light);

        app.register_type::<CameraSnap>();
    }
}

fn orbit_camera(
    q_camera_targets: Query<
        (&PlayerType, &TargetAction),
        With<CameraTarget>,
    >,
    mut q_orbit_a: QueryCameraA<(
        &OrbitCamera,
        &mut OrbitCameraAngle,
    )>,
    mut q_orbit_b: QueryCameraB<(
        &OrbitCamera,
        &mut OrbitCameraAngle,
    )>,
    q_actions: Query<&ActionState<PlayerAction>>,
    time: Res<Time>,
) -> Result {
    let dt = time.delta_secs();

    for (camera_type, target_action) in q_camera_targets.iter() {
        let (orbit, mut angle) = match camera_type {
            PlayerType::A => q_orbit_a.single_mut(),
            PlayerType::B => q_orbit_b.single_mut(),
        }?;

        let action = q_actions.get(target_action.get())?;

        let aim = action.axis_pair(&PlayerAction::Aim);

        angle.yaw_angle -= aim.x * orbit.sensitivity * dt;
        angle.pitch_angle -= aim.y * orbit.sensitivity * dt;
    }

    Ok(())
}

fn snap_camera(
    mut q_camera_a: QueryCameraA<&mut Transform, With<Camera>>,
    mut q_camera_b: QueryCameraB<&mut Transform, With<Camera>>,
    mut q_camera_full: QueryCameraFull<&mut Transform>,
    q_camera_snaps: Query<
        (&GlobalTransform, &CameraType),
        (
            Or<(
                Added<CameraSnap>,
                Changed<GlobalTransform>,
                Added<CameraType>,
            )>,
            With<CameraSnap>,
        ),
    >,
) -> Result {
    if q_camera_snaps.is_empty() {
        // Nothing to snap.
        return Ok(());
    }

    let mut camera_a = q_camera_a.single_mut()?;
    let mut camera_b = q_camera_b.single_mut()?;
    let mut camera_full = q_camera_full.single_mut()?;

    for (snap_global_transform, camera_type) in q_camera_snaps.iter()
    {
        let target_transform =
            snap_global_transform.compute_transform();

        match camera_type {
            CameraType::Full => *camera_full = target_transform,
            CameraType::A => *camera_a = target_transform,
            CameraType::B => *camera_b = target_transform,
        }

        info!("Snapped camera of type: {camera_type:?}");
    }

    Ok(())
}

// TODO: Move to another script.
fn setup_directional_light(
    trigger: Trigger<OnAdd, DirectionalLight>,
    mut q_lights: Query<&mut DirectionalLight>,
) -> Result {
    let mut light = q_lights.get_mut(trigger.target())?;
    light.shadows_enabled = true;

    Ok(())
}

#[derive(Component, Reflect)]
#[require(RequireAction)]
#[reflect(Component)]
pub struct CameraTarget;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FollowCamera {
    pub distance: f32,
    /// The follow speed.
    pub speed: f32,
}

#[derive(Component, Reflect)]
#[require(OrbitCameraAngle)]
#[reflect(Component)]
pub struct OrbitCamera {
    pub sensitivity: f32,
}

#[derive(Component, Default)]
pub struct OrbitCameraAngle {
    pub yaw_angle: f32,
    pub pitch_angle: f32,
}

/// Snaps camera to the [`GlobalTransform`] of this entity on [add][Added].
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraSnap;
