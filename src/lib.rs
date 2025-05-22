use avian3d::prelude::*;
use bevy::core_pipeline::Skybox;
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::smaa::Smaa;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::pbr::ScreenSpaceAmbientOcclusion;
use bevy::prelude::*;

mod action;
mod interaction;
mod movement;
mod physics;
mod ui;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            // PhysicsPickingPlugin,
            PhysicsDebugPlugin::default(),
            bevy_skein::SkeinPlugin::default(),
            ui::UiPlugin,
            physics::PhysicsPlugin,
            movement::MovementPlugin,
            interaction::InteractionPlugin,
        ))
        .add_systems(Startup, setup_camera_and_environment)
        .add_observer(setup_directional_light);

        #[cfg(feature = "dev")]
        app.add_plugins((
            bevy_inspector_egui::bevy_egui::EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        ));
    }
}

fn setup_camera_and_environment(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    const INITIAL_FOCUS: Vec3 = Vec3::new(0.0, 3.0, 0.0);

    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..default()
        },
        Tonemapping::BlenderFilmic,
        Bloom::NATURAL,
        Transform::from_xyz(-3.5, 10.0, -15.0)
            .looking_at(INITIAL_FOCUS, Vec3::Y),
        DebandDither::Enabled,
        Msaa::Off,
        ScreenSpaceAmbientOcclusion::default(),
        Smaa::default(),
        Skybox {
            image: asset_server.load("pisa_diffuse_rgb9e5_zstd.ktx2"),
            brightness: 1000.0,
            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: asset_server
                .load("pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server
                .load("pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 1000.0,
            ..default()
        },
    ));
}

fn setup_directional_light(
    trigger: Trigger<OnAdd, DirectionalLight>,
    mut q_lights: Query<&mut DirectionalLight>,
) -> Result {
    let mut light = q_lights.get_mut(trigger.target())?;
    light.shadows_enabled = true;

    Ok(())
}
