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
                    update_grounded,
                    apply_gravity,
                    movement,
                    apply_movement_damping,
                )
                    .chain(),
            )
            .add_systems(
                PhysicsSchedule,
                kinematic_controller_collisions
                    .in_set(NarrowPhaseSet::Last),
            );

        app.register_type::<CharacterController>()
            .register_type::<Grounded>()
            .register_type::<MovementAcceleration>()
            .register_type::<MovementDampingFactor>()
            .register_type::<JumpImpulse>()
            .register_type::<ControllerGravity>()
            .register_type::<MaxSlopeAngle>();
    }
}

// /// Components for movement parameters
// #[derive(Bundle)]
// pub struct MovementBundle {
//     pub acceleration: MovementAcceleration,
//     pub damping: MovementDampingFactor,
//     pub jump_impulse: JumpImpulse,
//     pub max_slope_angle: MaxSlopeAngle,
// }

// impl MovementBundle {
//     pub const fn new(
//         acc: Scalar,
//         damp: Scalar,
//         jump: Scalar,
//         slope: Scalar,
//     ) -> Self {
//         Self {
//             acceleration: MovementAcceleration(acc),
//             damping: MovementDampingFactor(damp),
//             jump_impulse: JumpImpulse(jump),
//             max_slope_angle: MaxSlopeAngle(slope),
//         }
//     }
// }

// impl Default for MovementBundle {
//     fn default() -> Self {
//         Self::new(50.0, 0.9, 4.0, std::f32::consts::PI * 0.45)
//     }
// }

// /// Bundle grouping all necessary character controller components
// #[derive(Bundle)]
// pub struct CharacterControllerBundle {
//     pub controller: CharacterController,
//     pub body: RigidBody,
//     pub collider: Collider,
//     pub ground_caster: ShapeCaster,
//     pub gravity: ControllerGravity,
//     pub movement: MovementBundle,
// }

// impl CharacterControllerBundle {
//     pub fn new(collider: Collider, gravity: Vector) -> Self {
//         let mut caster_shape = collider.clone();
//         caster_shape.set_scale(Vector::new(1.1, 0.5, 1.1), 10);

//         Self {
//             controller: CharacterController,
//             body: RigidBody::Kinematic,
//             collider,
//             ground_caster: ShapeCaster::new(
//                 caster_shape,
//                 Vector::ZERO,
//                 Quaternion::default(),
//                 Dir3::NEG_Y,
//             )
//             .with_max_distance(1.5),
//             gravity: ControllerGravity(gravity),
//             movement: MovementBundle::default(),
//         }
//     }
// }

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
// -- Initialization For Testing (TODO: Move to other place) -----------------------------------------

/// Spawn a simple red cube using the character controller bundle
// fn spawn_character(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     commands.spawn((
//         Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
//         MeshMaterial3d(materials.add(Color::srgb_u8(255, 0, 0))),
//         Transform::from_xyz(0.0, 2.0, 0.0),
//         CharacterControllerBundle::new(
//             Collider::cuboid(1.0, 1.0, 1.0),
//             Vector::NEG_Y * 9.81,
//         ),
//     ));
// }

// /// Spawn a static ground plate so the character has something to move on
// fn spawn_plate(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let mesh = meshes.add(Cuboid::new(100.0, 0.1, 100.0));
//     let material = materials.add(Color::srgb_u8(0, 255, 0));

//     commands.spawn((
//         Mesh3d(mesh),
//         MeshMaterial3d(material),
//         Transform::from_xyz(0.0, 0.0, 0.0),
//         RigidBody::Static,
//         Collider::cuboid(50.0, 2.0, 50.0),
//     ));
// }

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

/// Updates grounded state by casting a small shape downward
fn update_grounded(
    mut commands: Commands,
    mut query: Query<
        (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
        With<CharacterController>,
    >,
) {
    for (entity, hits, rotation, slope) in query.iter_mut() {
        let grounded = hits.iter().any(|hit| {
            let normal = if hit.normal2.y > 0.0 {
                hit.normal2
            } else {
                -hit.normal2
            };

            if let Some(MaxSlopeAngle(max)) = slope {
                let angle = (rotation * normal)
                    .angle_between(Vector::Y)
                    .abs();
                angle <= *max
            } else {
                // Consider ground if normal points mostly up
                normal.y > 0.5
            }
        });

        if grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Applies gravity to vertical velocity
fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(
        &ControllerGravity,
        &mut LinearVelocity,
        Option<&Grounded>,
    )>,
) {
    let delta = time.delta_secs_f64().adjust_precision();
    for (gravity, mut vel, grounded) in query.iter_mut() {
        // Only apply gravity when not grounded
        if grounded.is_none() {
            vel.0.y += gravity.0.y * delta;
            let max_fall_speed = -20.0;
            vel.0.y = vel.0.y.max(max_fall_speed);
        }
    }
}

/// Handles movement and jumping
fn movement(
    time: Res<Time>,
    mut reader: EventReader<MovementAction>,
    cam_tf_q: Query<&GlobalTransform, With<Camera3d>>,
    mut query: Query<
        (
            &MovementAcceleration,
            &JumpImpulse,
            &mut LinearVelocity,
            Option<&Grounded>,
            &mut Transform,
        ),
        With<CharacterController>,
    >,
) {
    let dt = time.delta_secs_f64().adjust_precision();

    // Speed caps
    let max_walk = 5.0;
    let max_sprint = 10.0;

    // Get camera yaw from its GlobalTransform
    let cam_tf = match cam_tf_q.single() {
        Ok(tf) => tf,
        Err(_) => return,
    };
    // Extract yaw (rotation around Y) via Euler YXZ sequence
    let (cam_yaw, _, _) = cam_tf.rotation().to_euler(EulerRot::YXZ);
    // Track whether any WASD event occurred
    let mut did_move = false;

    for event in reader.read() {
        match event {
            MovementAction::Move { dir, sprint } => {
                // skip zero input
                if *dir == Vector2::ZERO {
                    continue;
                }
                did_move = true;

                // Compute input angle: atan2(right, forward)
                let input_angle = f32::atan2(-dir.x, dir.y);

                // Final yaw = camera_yaw + input_angle
                let yaw = cam_yaw + input_angle;

                for (acc, _jump, mut vel, _gd, mut tx) in
                    query.iter_mut()
                {
                    // Rotate the cube to face yaw
                    tx.rotation = Quat::from_rotation_y(yaw);

                    // Move along its local -Z
                    let forward = tx
                        .rotation
                        .mul_vec3(Vec3::NEG_Z)
                        .normalize_or_zero();

                    // Sprint factor
                    let sprint_factor =
                        if *sprint { 2.0 } else { 1.0 };

                    // Apply acceleration
                    vel.0 += forward * (acc.0 * dt * sprint_factor);

                    // Clamp horizontal speed
                    let max_speed =
                        if *sprint { max_sprint } else { max_walk };
                    let horiz = Vec2::new(vel.0.x, vel.0.z);
                    if horiz.length() > max_speed {
                        let clamped = horiz.normalize() * max_speed;
                        vel.0.x = clamped.x;
                        vel.0.z = clamped.y;
                    }
                }
            }
            MovementAction::Jump => {
                for (_acc, jump, mut vel, grounded, _tx) in
                    query.iter_mut()
                {
                    if grounded.is_some() {
                        vel.0.y = jump.0;
                    }
                }
            }
        }
    }

    // If no movement this frame, face camera yaw
    if !did_move {
        let cam_quat = Quat::from_rotation_y(cam_yaw);
        for (_acc, _j, _vel, _gd, mut tx) in query.iter_mut() {
            tx.rotation = cam_quat;
        }
    }
}

/// Applies damping to horizontal movement
fn apply_movement_damping(
    mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>,
) {
    for (damp, mut vel) in query.iter_mut() {
        vel.x *= damp.0;
        vel.z *= damp.0;
    }
}

/// Handles collisions for kinematic character controllers
fn kinematic_controller_collisions(
    collisions: Collisions,
    bodies: Query<&RigidBody>,
    colliders: Query<&ColliderOf>,
    mut query: Query<
        (&mut Position, &mut LinearVelocity, Option<&MaxSlopeAngle>),
        With<CharacterController>,
    >,
) {
    for contacts in collisions.iter() {
        let Ok(
            [&ColliderOf { body: rb1 }, &ColliderOf { body: rb2 }],
        ) = colliders
            .get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        // Determine which entity is the character controller
        let (controller_entity, controller_is_first) =
            if query.contains(rb1) {
                (rb1, true)
            } else if query.contains(rb2) {
                (rb2, false)
            } else {
                continue;
            };

        // Only handle kinematic bodies against static geometry
        if !bodies.get(controller_entity).unwrap().is_kinematic() {
            continue;
        }

        // Get mutable references to position and velocity
        let (mut pos, mut vel, _slope) =
            query.get_mut(controller_entity).unwrap();

        for manifold in contacts.manifolds.iter() {
            // Make sure normal points toward the character if it's the second collider
            let normal = if controller_is_first {
                manifold.normal
            } else {
                -manifold.normal
            };

            // Calculate total penetration to resolve
            let mut max_pen = 0.0;
            for point in manifold.points.iter() {
                if point.penetration > max_pen {
                    max_pen = point.penetration;
                }
            }

            if max_pen > 0.0 {
                // Apply resolution with a small buffer to prevent oscillation
                let resolution_vector = normal * (max_pen + 0.001);
                pos.0 += resolution_vector;

                // Determine collision response based on normal direction

                // Floor collision (normal pointing up)
                if normal.y > 0.7 {
                    // Zero out downward velocity on floor
                    if vel.y < 0.0 {
                        vel.y = 0.0;
                    }
                }
                // Ceiling collision (normal pointing down)
                else if normal.y < -0.7 {
                    // Zero out upward velocity on ceiling
                    if vel.y > 0.0 {
                        vel.y = 0.0;
                    }
                }
                // Wall collision (normal mostly horizontal)
                else {
                    // Project velocity onto the normal
                    let vel_into_normal = vel.0.dot(normal);

                    // Only cancel velocity going into the wall
                    if vel_into_normal < 0.0 {
                        // Remove velocity component going into the wall
                        vel.0 -= normal * vel_into_normal;
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
pub struct CharacterController;

/// Marker for grounded state
#[derive(Component, Reflect)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
pub struct Grounded;

/// Acceleration magnitude for movement
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementAcceleration(pub Scalar);

/// Damping factor to slow XZ movement
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementDampingFactor(pub Scalar);

/// Impulse strength for jumps
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct JumpImpulse(pub Scalar);

/// Gravity applied to the controller
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ControllerGravity(pub Vector);

/// Maximum climbable slope angle
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MaxSlopeAngle(pub Scalar);
