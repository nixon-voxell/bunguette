use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use leafwing_input_manager::prelude::*;
use split_screen::{CameraType, QueryCameras};

use crate::action::{PlayerAction, RequireAction, TargetAction};
use crate::asset_pipeline::CurrentScene;
use crate::player::PlayerType;

pub mod split_screen;

pub const UI_RENDER_LAYER: RenderLayers = RenderLayers::layer(1);

pub(super) struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(split_screen::SplitScreenPlugin);

        app.add_systems(
            PostUpdate,
            (
                third_person_camera,
                snap_camera,
                setup_third_person_camera,
            )
                .chain()
                .after(TransformSystem::TransformPropagate),
        )
        .add_observer(setup_directional_light);

        app.register_type::<CameraSnap>()
            .register_type::<ThirdPersonCamera>()
            .register_type::<CameraTarget>();
    }
}

fn third_person_camera(
    q_camera_targets: Query<
        (&PlayerType, &GlobalTransform, &TargetAction),
        With<CameraTarget>,
    >,
    mut q_cameras: QueryCameras<
        (&ThirdPersonCamera, &mut OrbitAngle, &mut Transform),
        With<CameraSnap>,
    >,
    q_actions: Query<(
        &ActionState<PlayerAction>,
        &InputMap<PlayerAction>,
    )>,
    time: Res<Time>,
) -> Result {
    let dt = time.delta_secs();

    for (camera_type, target_transform, target_action) in
        q_camera_targets.iter()
    {
        let (config, mut angle, mut camera_transform) =
            match camera_type {
                PlayerType::A => q_cameras.get_mut(CameraType::A),
                PlayerType::B => q_cameras.get_mut(CameraType::B),
            }?;

        let (action, input_map) =
            q_actions.get(target_action.get())?;

        let is_gamepad = input_map.gamepad().is_some();
        let aim = action.axis_pair(&PlayerAction::Aim);

        // Gamepad gets a boost in sensitivity.
        let device_sensitivity = if is_gamepad { 10.0 } else { 1.0 };

        let mut aim_y = aim.y
            * config.pitch_sensitivity
            * device_sensitivity
            * dt;

        aim_y = if is_gamepad { -aim_y } else { aim_y };

        angle.yaw -=
            aim.x * config.yaw_sensitivity * device_sensitivity * dt;
        angle.pitch += aim_y;

        // Clamp pitch to prevent camera flipping overhead or underfoot.
        angle.pitch =
            angle.pitch.clamp(0.0, FRAC_PI_2 * config.max_pitch);

        // Keep yaw within 0 to 2*PI range for consistency,
        // though not strictly necessary due to trigonometric
        // functions handling periodicity.
        angle.yaw = angle.yaw.rem_euclid(TAU);

        let focus = target_transform.translation();
        let current_distance =
            focus.distance(camera_transform.translation);
        let distance = current_distance
            .lerp(config.distance, dt * config.follow_speed);

        // Calculate camera position using spherical coordinates logic
        let cam_x =
            focus.x + distance * angle.pitch.cos() * angle.yaw.sin();
        let cam_y = focus.y + distance * angle.pitch.sin();
        let cam_z =
            focus.z + distance * angle.pitch.cos() * angle.yaw.cos();

        camera_transform.translation = Vec3::new(cam_x, cam_y, cam_z);
        camera_transform.look_at(focus, Vec3::Y);
    }

    Ok(())
}

fn snap_camera(
    mut q_cameras: QueryCameras<&mut Transform>,
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

    for (snap_global_transform, camera_type) in q_camera_snaps.iter()
    {
        let target_transform =
            snap_global_transform.compute_transform();

        let mut camera_transform = q_cameras.get_mut(*camera_type)?;
        *camera_transform = target_transform;

        debug!("Snapped camera of type: {camera_type:?}");
    }

    Ok(())
}

/// Copy parent transform and clear the replace the parent
/// with the [`CurrentScene`]'s entity!
fn setup_third_person_camera(
    mut commands: Commands,
    q_transforms: Query<&Transform, Without<ThirdPersonCamera>>,
    mut q_cameras: Query<
        (&ChildOf, &mut Transform, Entity),
        Added<ThirdPersonCamera>,
    >,
    current_scene: Res<CurrentScene>,
) -> Result {
    for (child_of, mut transform, entity) in q_cameras.iter_mut() {
        let parent_transform = q_transforms.get(child_of.parent())?;
        *transform = *parent_transform;

        commands.entity(entity).insert(ChildOf(
            current_scene
                .get()
                .ok_or("Should have a scene loaded!")?,
        ));
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

/// Snaps camera to the [`GlobalTransform`] of this entity on [add][Added].
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraSnap;

#[derive(Component, Reflect)]
#[require(OrbitAngle)]
#[reflect(Component, Default)]
pub struct ThirdPersonCamera {
    /// The yaw angle sensitivity.
    pub yaw_sensitivity: f32,
    /// The pitch angle sensitivity.
    pub pitch_sensitivity: f32,
    /// The distance between the
    /// camera and the [`CameraTarget`].
    pub distance: f32,
    /// The follow speed.
    pub follow_speed: f32,
    /// Max pitch angle in percentage from 0 - 1.
    /// Will be multiplied by [`FRAC_PI_2`].
    pub max_pitch: f32,
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            yaw_sensitivity: 0.4,
            pitch_sensitivity: 0.4,
            distance: 4.0,
            follow_speed: 10.0,
            max_pitch: 0.8,
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct OrbitAngle {
    pub yaw: f32,
    pub pitch: f32,
}
