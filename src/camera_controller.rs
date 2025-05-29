use std::f32::consts::{FRAC_PI_2, TAU};

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
        .add_systems(Update, third_person_camera)
        .add_observer(setup_directional_light);

        app.register_type::<CameraSnap>()
            .register_type::<ThirdPersonCamera>();
    }
}

fn third_person_camera(
    q_camera_targets: Query<
        (&PlayerType, &GlobalTransform, &TargetAction),
        With<CameraTarget>,
    >,
    mut q_orbit_a: QueryCameraA<(
        &ThirdPersonCamera,
        &mut OrbitAngle,
        &mut Transform,
    )>,
    mut q_orbit_b: QueryCameraB<(
        &ThirdPersonCamera,
        &mut OrbitAngle,
        &mut Transform,
    )>,
    q_actions: Query<&ActionState<PlayerAction>>,
    time: Res<Time>,
) -> Result {
    let dt = time.delta_secs();

    for (camera_type, target_transform, target_action) in
        q_camera_targets.iter()
    {
        let (config, mut angle, mut camera_transform) =
            match camera_type {
                PlayerType::A => q_orbit_a.single_mut(),
                PlayerType::B => q_orbit_b.single_mut(),
            }?;

        let action = q_actions.get(target_action.get())?;

        let aim = action.axis_pair(&PlayerAction::Aim);

        angle.yaw -= aim.x * config.sensitivity * dt;
        angle.pitch -= aim.y * config.sensitivity * dt;

        // Clamp pitch to prevent camera flipping overhead or underfoot.
        angle.pitch = angle.pitch.clamp(
            -FRAC_PI_2 + 0.05, // Slightly > -90 degrees
            FRAC_PI_2 - 0.05,  // Slightly <  90 degrees
        );

        // Keep yaw within 0 to 2*PI range for consistency,
        // though not strictly necessary due to trigonometric
        // functions handling periodicity.
        angle.yaw = angle.yaw.rem_euclid(TAU);

        let focus = target_transform.translation();

        // Calculate camera position using spherical coordinates logic
        let cam_x = focus.x
            + config.distance * angle.pitch.cos() * angle.yaw.sin();
        let cam_y = focus.y + config.distance * angle.pitch.sin();
        let cam_z = focus.z
            + config.distance * angle.pitch.cos() * angle.yaw.cos();

        camera_transform.translation = Vec3::new(cam_x, cam_y, cam_z);
        camera_transform.look_at(focus, Vec3::Y);
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
#[require(OrbitAngle)]
#[reflect(Component)]
pub struct ThirdPersonCamera {
    /// The orbit sensitivity.
    pub sensitivity: f32,
    /// The distance between the
    /// camera and the [`CameraTarget`].
    pub distance: f32,
    /// The follow speed.
    pub speed: f32,
}

#[derive(Component, Default)]
pub struct OrbitAngle {
    pub yaw: f32,
    pub pitch: f32,
}

/// Snaps camera to the [`GlobalTransform`] of this entity on [add][Added].
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraSnap;
