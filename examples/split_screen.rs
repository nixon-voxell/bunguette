use std::f32::consts::PI;

use bevy::{
    core_pipeline::{
        bloom::Bloom,
        smaa::Smaa,
        tonemapping::{DebandDither, Tonemapping},
    },
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
    render::camera::Viewport,
    window::WindowResized,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (set_camera_viewports, button_system))
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(
            meshes.add(Plane3d::default().mesh().size(100.0, 100.0)),
        ),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Light
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -PI / 4.,
        )),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
    ));

    // Cameras and their dedicated UI
    for (index, (camera_name, camera_pos)) in [
        ("Player 1", Vec3::new(0.0, 200.0, -150.0)),
        ("Player 2", Vec3::new(150.0, 150., 50.0)),
        ("Player 3", Vec3::new(100.0, 150., -150.0)),
        ("Player 4", Vec3::new(-100.0, 80., 150.0)),
    ]
    .iter()
    .enumerate()
    {
        let camera = commands
            .spawn((
                Camera3d::default(),
                Transform::from_translation(*camera_pos)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                Camera {
                    // Renders cameras with different priorities to prevent ambiguities
                    order: index as isize,
                    hdr: true,
                    ..default()
                },
                Msaa::Off,
                ScreenSpaceAmbientOcclusion::default(),
                Smaa::default(),
                Tonemapping::None,
                Bloom::NATURAL,
                DebandDither::Enabled,
                CameraPosition {
                    pos: UVec2::new(
                        (index % 2) as u32,
                        (index / 2) as u32,
                    ),
                },
            ))
            .id();

        // Set up UI
        commands
            .spawn((
                UiTargetCamera(camera),
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(*camera_name),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(12.),
                        left: Val::Px(12.),
                        ..default()
                    },
                ));
                buttons_panel(parent);
            });
    }

    commands.spawn((
        Camera2d,
        Camera {
            order: 10,
            hdr: true,
            ..default()
        },
        // ScreenSpaceAmbientOcclusion::default(),
        // Smaa::default(),
        // Msaa::Off,
        IsDefaultUiCamera,
    ));

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        Children::spawn(Spawn((
            Node {
                width: Val::Percent(10.0),
                height: Val::Percent(10.0),
                ..default()
            },
            BackgroundColor(Color::WHITE),
            BoxShadow::new(
                Color::BLACK,
                Val::Px(4.0),
                Val::Px(4.0),
                Val::Px(12.0),
                Val::Px(8.0),
            ),
        ))),
    ));

    fn buttons_panel(parent: &mut ChildSpawnerCommands) {
        parent
            .spawn(Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.)),
                ..default()
            })
            .with_children(|parent| {
                rotate_button(parent, "<", Direction::Left);
                rotate_button(parent, ">", Direction::Right);
            });
    }

    fn rotate_button(
        parent: &mut ChildSpawnerCommands,
        caption: &str,
        direction: Direction,
    ) {
        parent
            .spawn((
                RotateCamera(direction),
                Button,
                Node {
                    width: Val::Px(40.),
                    height: Val::Px(40.),
                    border: UiRect::all(Val::Px(2.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(Color::WHITE),
                BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            ))
            .with_children(|parent| {
                parent.spawn(Text::new(caption));
            });
    }
}

#[derive(Component)]
struct CameraPosition {
    pos: UVec2,
}

#[derive(Component)]
struct RotateCamera(Direction);

enum Direction {
    Left,
    Right,
}

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<(&CameraPosition, &mut Camera)>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.read() {
        let window = windows.get(resize_event.window).unwrap();
        let size = window.physical_size() / 2;

        for (camera_position, mut camera) in &mut query {
            camera.viewport = Some(Viewport {
                physical_position: camera_position.pos * size,
                physical_size: size,
                ..default()
            });
        }
    }
}

fn button_system(
    interaction_query: Query<
        (&Interaction, &ComputedNodeTarget, &RotateCamera),
        (Changed<Interaction>, With<Button>),
    >,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    for (interaction, computed_target, RotateCamera(direction)) in
        &interaction_query
    {
        if let Interaction::Pressed = *interaction {
            // Since TargetCamera propagates to the children, we can use it to find
            // which side of the screen the button is on.
            if let Some(mut camera_transform) = computed_target
                .camera()
                .and_then(|camera| camera_query.get_mut(camera).ok())
            {
                let angle = match direction {
                    Direction::Left => -0.1,
                    Direction::Right => 0.1,
                };
                camera_transform.rotate_around(
                    Vec3::ZERO,
                    Quat::from_axis_angle(Vec3::Y, angle),
                );
            }
        }
    }
}
