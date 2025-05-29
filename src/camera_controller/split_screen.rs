use bevy::color::palettes::css::SKY_BLUE;
use bevy::core_pipeline::Skybox;
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::core_3d::Camera3dDepthLoadOp;
use bevy::core_pipeline::smaa::Smaa;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::ecs::component::{ComponentHooks, Immutable, StorageType};
use bevy::prelude::*;
use bevy::render::camera::{CameraOutputMode, Viewport};
use bevy::render::view::RenderLayers;
use bevy::window::WindowResized;

use super::UI_RENDER_LAYER;

pub(super) struct SplitScreenPlugin;

impl Plugin for SplitScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_camera_and_environment)
            .add_systems(Update, set_camera_split_viewports);

        app.register_type::<CameraType>()
            .register_type::<CameraSnap>();
    }
}

fn set_camera_split_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut q_camera_a: QueryCameraA<&mut Camera>,
    mut q_camera_b: QueryCameraB<&mut Camera>,
) -> Result {
    // We need to dynamically resize the camera's viewports whenever the
    // window size changes so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created,
    // allowing us to reuse this system for initial setup.

    for resize_event in resize_events.read() {
        let window_size =
            windows.get(resize_event.window).unwrap().physical_size();
        let additional_pixel = window_size.x % 2;
        let split_size = UVec2::new(window_size.x / 2, window_size.y);

        let mut camera_a = q_camera_a.single_mut()?;
        let mut camera_b = q_camera_b.single_mut()?;

        camera_a.viewport = Some(Viewport {
            physical_position: UVec2::ZERO,
            physical_size: split_size,
            ..default()
        });
        camera_b.viewport = Some(Viewport {
            physical_position: UVec2::new(split_size.x, 0),
            physical_size: split_size
                + UVec2::new(additional_pixel, 0),
            ..default()
        });
    }

    Ok(())
}

fn setup_camera_and_environment(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Spawn a camera with clear color.
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(SKY_BLUE.into()),
            output_mode: CameraOutputMode::Skip,
            ..default()
        },
        Msaa::Off,
        // Use a layer that no one uses.
        RenderLayers::layer(31),
    ));

    commands
        .spawn((game_camera_bundle(&asset_server, 0), CameraType::A));

    commands
        .spawn((game_camera_bundle(&asset_server, 1), CameraType::B));

    commands.spawn((ui_camera_bundle(2), CameraType::Full));
}

fn game_camera_bundle(
    asset_server: &AssetServer,
    order: isize,
) -> impl Bundle {
    let diffuse_map =
        asset_server.load("pisa_diffuse_rgb9e5_zstd.ktx2");
    let specular_map =
        asset_server.load("pisa_specular_rgb9e5_zstd.ktx2");

    (
        Camera3d {
            depth_load_op: Camera3dDepthLoadOp::Load,
            ..default()
        },
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::None,
            order,
            output_mode: CameraOutputMode::Skip,
            ..default()
        },
        Tonemapping::None,
        Msaa::Off,
        Skybox {
            image: diffuse_map.clone(),
            brightness: 1000.0,
            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: diffuse_map.clone(),
            specular_map: specular_map.clone(),
            intensity: 1000.0,
            ..default()
        },
    )
}

fn ui_camera_bundle(order: isize) -> impl Bundle {
    (
        Camera3d {
            depth_load_op: Camera3dDepthLoadOp::Load,
            ..default()
        },
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::None,
            order,
            ..default()
        },
        Msaa::Off,
        Smaa::default(),
        Tonemapping::None,
        Bloom::NATURAL,
        DebandDither::Enabled,
        IsDefaultUiCamera,
        UI_RENDER_LAYER,
    )
}

/// Snaps camera to the [`GlobalTransform`] of this entity on [add][Added].
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraSnap;

#[derive(Reflect, Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[reflect(Component)]
pub enum CameraType {
    Full,
    A,
    B,
}

impl Component for CameraType {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    type Mutability = Immutable;

    /// Setup camera tag: [`CameraFull`], [`CameraA`], or [`CameraB`]
    /// based on [`CameraType`].
    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, hook| {
            let entity = hook.entity;
            let camera_type = world.get::<Self>(hook.entity).unwrap();

            match camera_type {
                CameraType::Full => {
                    world
                        .commands()
                        .entity(entity)
                        .insert(CameraFull);
                }
                CameraType::A => {
                    world.commands().entity(entity).insert(CameraA);
                }
                CameraType::B => {
                    world.commands().entity(entity).insert(CameraB);
                }
            }
        });
    }
}

/// A unique query to the [`CameraA`] entity.
#[allow(dead_code)]
pub type QueryCameraA<'w, 's, D, F = ()> = Query<
    'w,
    's,
    D,
    (
        F,
        With<CameraA>,
        Without<CameraB>,
        Without<CameraFull>,
        With<Camera>,
    ),
>;

/// A unique query to the [`CameraB`] entity.
#[allow(dead_code)]
pub type QueryCameraB<'w, 's, D, F = ()> = Query<
    'w,
    's,
    D,
    (
        F,
        With<CameraB>,
        Without<CameraA>,
        Without<CameraFull>,
        With<Camera>,
    ),
>;

/// A unique query to the [`CameraFull`] entity.
#[allow(dead_code)]
pub type QueryCameraFull<'w, 's, D, F = ()> = Query<
    'w,
    's,
    D,
    (
        F,
        With<CameraFull>,
        Without<CameraA>,
        Without<CameraB>,
        With<Camera>,
    ),
>;

/// A unique component for [`Camera`] that full covers the entire screen
/// and renders on top of [`CameraA`] & [`CameraB`].
///
/// Usually used for full screen ui.
#[derive(Component, Debug)]
pub struct CameraFull;

/// A unique component for [`Camera`] on the left side of the screen.
///
/// Usually used to render the POV of [`crate::player::PlayerA`]
#[derive(Component, Debug)]
pub struct CameraA;

/// A unique component for [`Camera`] on the right side of the screen.
///
/// Usually used to render the POV of [`crate::player::PlayerB`]
#[derive(Component, Debug)]
pub struct CameraB;
