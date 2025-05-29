use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use split_screen::{
    CameraSnap, CameraType, QueryCameraA, QueryCameraB,
    QueryCameraFull,
};

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
        .add_observer(setup_directional_light);
    }
}

fn snap_camera(
    mut q_camera_a: QueryCameraA<&mut Transform>,
    mut q_camera_b: QueryCameraB<&mut Transform>,
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
