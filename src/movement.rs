use avian3d::{math::*, prelude::*};
use bevy::prelude::*;

/// Plugin that sets up kinematic character movement for a red cube prototype
pub(super) struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_test_scene)
            .add_event::<MovementAction>()
            .add_systems(
                Update,
                (
                    keyboard_input,
                    apply_gravity,
                    movement,
                    apply_movement_damping,
                    update_grounded,
                )
                    .chain(),
            )
            .add_systems(
                PhysicsSchedule,
                kinematic_controller_collisions
                    .in_set(NarrowPhaseSet::Last),
            );

        app.register_type::<CharacterController>();
    }
}

// -- TESTING SCENE ----------------------------------------------------------------
fn spawn_test_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SceneRoot(
        asset_server.load(
            GltfAssetLabel::Scene(0)
                .from_asset("scenes/movement_test.glb"),
        ),
    ));
}

// -- Keyboard Input ----------------------------------------------------------------

/// Reads keyboard input and emits movement events
fn keyboard_input(
    mut writer: EventWriter<MovementAction>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let up = keys.pressed(KeyCode::KeyW);
    let down = keys.pressed(KeyCode::KeyS);
    let left = keys.pressed(KeyCode::KeyA);
    let right = keys.pressed(KeyCode::KeyD);

    let dir = Vector2::new(
        (right as i32 - left as i32) as Scalar,
        (up as i32 - down as i32) as Scalar,
    )
    .clamp_length_max(1.0);

    let sprint = keys.pressed(KeyCode::ShiftLeft)
        || keys.pressed(KeyCode::ShiftRight);

    if dir != Vector2::ZERO {
        writer.write(MovementAction::Move { dir, sprint });
    }
    if keys.just_pressed(KeyCode::Space) {
        writer.write(MovementAction::Jump);
    }
}

// -- Movement ----------------------------------------------------------------

/// Updates grounded state
fn update_grounded(
    mut query: Query<(
        &ShapeHits,
        &Rotation,
        &mut CharacterController,
    )>,
) {
    for (hits, rot, mut ctl) in query.iter_mut() {
        let on_ground = hits.iter().any(|hit| {
            let normal = if hit.normal2.y > 0.0 {
                hit.normal2
            } else {
                -hit.normal2
            };
            let angle = (rot * normal).angle_between(Vector::Y).abs();
            angle <= ctl.max_slope_angle
        });
        if on_ground {
            ctl.grounded = true;
            // Zero vertical velocity when grounded
            ctl.velocity.y = 0.0;
        } else {
            ctl.grounded = false;
        }
    }
}

/// Applies gravity to vertical velocity
fn apply_gravity(
    time: Res<Time>,
    mut query: Query<&mut CharacterController>,
) {
    let dt = time.delta_secs_f64().adjust_precision();
    for mut ctl in query.iter_mut() {
        if !ctl.grounded {
            let gravity = ctl.gravity;
            ctl.velocity += gravity * dt;
        }
    }
}

/// Handles movement and jumping
fn movement(
    time: Res<Time>,
    mut reader: EventReader<MovementAction>,
    cam_tf_q: Query<&GlobalTransform, With<Camera3d>>,
    mut query: Query<(&mut CharacterController, &mut Transform)>,
) {
    let dt = time.delta_secs_f64() as f32;

    // Speed caps
    let max_walk = 5.0;
    let max_sprint = 10.0;

    // Get camera transform
    let cam_tf = match cam_tf_q.single() {
        Ok(tf) => tf,
        Err(_) => return,
    };
    let cam_forward = cam_tf.forward();
    let cam_forward = Vec3::new(cam_forward.x, 0.0, cam_forward.z)
        .normalize_or_zero();
    let cam_right = cam_forward.cross(Vec3::Y).normalize_or_zero();

    for event in reader.read() {
        match event {
            MovementAction::Move { dir, sprint }
                if *dir != Vector2::ZERO =>
            {
                // Compute yaw directly from that vector: atan2(x, z)
                let world_move =
                    (cam_forward * dir.y) + (cam_right * dir.x);
                let world_move = world_move.normalize_or_zero();

                // Compute yaw and apply offset based on model orientation
                let yaw = f32::atan2(world_move.x, world_move.z);
                // The model is rotated 90 degrees (Facing -X)
                let offset = std::f32::consts::PI / 2.0;

                for (mut ctl, mut tx) in query.iter_mut() {
                    // Rotate to face movement direction
                    tx.rotation = Quat::from_rotation_y(yaw + offset);

                    // Apply acceleration * sprint factor
                    let factor = if *sprint { 2.0 } else { 1.0 };
                    let acceleration = ctl.acceleration;
                    ctl.velocity +=
                        world_move * (acceleration * dt * factor);

                    // Clamp horizontal speed
                    let max_speed =
                        if *sprint { max_sprint } else { max_walk };
                    let horiz =
                        Vec2::new(ctl.velocity.x, ctl.velocity.z);
                    if horiz.length() > max_speed {
                        let clamped = horiz.normalize() * max_speed;
                        ctl.velocity.x = clamped.x;
                        ctl.velocity.z = clamped.y;
                    }
                }
            }
            MovementAction::Jump => {
                for (mut ctl, _) in query.iter_mut() {
                    if ctl.grounded {
                        ctl.velocity.y = ctl.jump_impulse;
                        ctl.grounded = false;
                    }
                }
            }
            _ => {}
        }
    }

    // Translation
    for (ctl, mut tx) in query.iter_mut() {
        tx.translation += ctl.velocity * dt;
    }
}

/// Applies damping to horizontal movement
fn apply_movement_damping(
    mut query: Query<&mut CharacterController>,
) {
    for mut ctl in query.iter_mut() {
        ctl.velocity.x *= ctl.damping;
        ctl.velocity.z *= ctl.damping;
    }
}

/// Handles collisions for kinematic character controllers
fn kinematic_controller_collisions(
    collisions: Collisions,
    bodies: Query<&RigidBody>,
    colliders: Query<&ColliderOf>,
    mut query: Query<(&mut Position, &mut CharacterController)>,
) {
    for contacts in collisions.iter() {
        // Find the two bodies involved
        let Ok(
            [&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }],
        ) = colliders
            .get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        // Figure out which one is our controller
        let (controller_entity, controller_is_first) =
            if query.contains(rb1) {
                (rb1, true)
            } else if query.contains(rb2) {
                (rb2, false)
            } else {
                continue;
            };

        // Only handle kinematic controllers
        if !bodies.get(controller_entity).unwrap().is_kinematic() {
            continue;
        }

        // Grab the position and our merged CharacterController
        let (mut pos, mut ctl) =
            query.get_mut(controller_entity).unwrap();

        for manifold in &contacts.manifolds {
            // Ensure the normal always points into the controller
            let normal = if controller_is_first {
                manifold.normal
            } else {
                -manifold.normal
            };

            // Find the deepest penetration
            let max_pen = manifold
                .points
                .iter()
                .map(|pt| pt.penetration)
                .fold(0.0, f32::max);

            if max_pen > 0.0 {
                // Resolve penetration
                pos.0 += normal * (max_pen + 0.001);

                // Floor hit? zero downward velocity
                if normal.y > 0.7 {
                    if ctl.velocity.y < 0.0 {
                        ctl.velocity.y = 0.0;
                    }
                }
                // Ceiling hit? zero upward velocity
                else if normal.y < -0.7 {
                    if ctl.velocity.y > 0.0 {
                        ctl.velocity.y = 0.0;
                    }
                }
                // Wall slide: remove velocity into the wall
                else {
                    let into = ctl.velocity.dot(normal);
                    if into < 0.0 {
                        ctl.velocity -= normal * into;
                    }
                }
            }
        }
    }
}

/// Movement actions triggered by input
#[derive(Event)]
pub enum MovementAction {
    Move { dir: Vector2, sprint: bool },
    Jump,
}

/// Marker for kinematic character bodies
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CharacterController {
    pub acceleration: Scalar,
    pub damping: Scalar,
    pub jump_impulse: Scalar,
    pub max_slope_angle: Scalar,
    // Gravity
    pub gravity: Vector,
    // State - Compute only
    pub grounded: bool,
    pub velocity: Vector,
}
