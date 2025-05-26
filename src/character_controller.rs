use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    action::{PlayerAction, RequireAction, TargetAction},
    player::PlayerType,
};

/// Plugin that sets up kinematic character movement
pub(super) struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_grounded,
                apply_gravity,
                movement,
                jump,
                rotate_to_velocity,
                movement_damping,
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

/// Check grounded state by raycasting downwards.
fn check_grounded(
    mut q_characters: Query<(
        Entity,
        &GlobalTransform,
        &CharacterController,
        &mut IsGrounded,
    )>,
    spatial_query: SpatialQuery,
) {
    const MAX_DIST: f32 = 0.2;
    const SHAPE_CAST_CONFIG: ShapeCastConfig =
        ShapeCastConfig::from_max_distance(MAX_DIST);

    let shape = Collider::sphere(0.2);

    for (entity, global_transform, character, mut is_grounded) in
        q_characters.iter_mut()
    {
        let char_pos = global_transform.translation();

        let ray_origin = char_pos;
        let ray_direction = Dir3::NEG_Y;
        // let max_distance = 1.0;

        // Exclude the character's own entity from the raycast
        let filter = SpatialQueryFilter::default()
            .with_excluded_entities([entity]);

        if let Some(hit) = spatial_query.cast_shape(
            &shape,
            ray_origin,
            Quat::IDENTITY,
            ray_direction,
            &SHAPE_CAST_CONFIG,
            &filter,
        ) {
            let slope_angle = hit.normal1.angle_between(Vec3::Y);

            // Check if the normal is valid and surface is walkable
            if slope_angle.is_finite() {
                is_grounded.0 =
                    slope_angle <= character.max_slope_angle;
            } else {
                is_grounded.0 = false;
            }
        } else {
            is_grounded.0 = false;
        }
    }
}

fn jump(
    mut q_characters: Query<(
        &mut LinearVelocity,
        &mut IsGrounded,
        &CharacterController,
        &TargetAction,
    )>,
    q_actions: Query<&ActionState<PlayerAction>>,
) {
    for (
        mut linear_velocity,
        mut is_grounded,
        character,
        target_action,
    ) in q_characters.iter_mut()
    {
        let Ok(action) = q_actions.get(target_action.get()) else {
            continue;
        };
        info!("jumpable...");

        if is_grounded.0 && action.just_pressed(&PlayerAction::Jump) {
            info!("jump!");
            linear_velocity.0.y = character.jump_impulse;
            is_grounded.0 = false;
        }
    }
}

fn rotate_to_velocity(
    mut q_characters: Query<
        (&mut Rotation, &LinearVelocity, &TargetAction),
        With<CharacterController>,
    >,
    q_actions: Query<&ActionState<PlayerAction>>,
    time: Res<Time>,
) {
    const ROTATION_RATE: f32 = 10.0;
    let dt = time.delta_secs();

    for (mut rotation, linear_velocity, target_action) in
        q_characters.iter_mut()
    {
        let Ok(action) = q_actions.get(target_action.get()) else {
            continue;
        };

        // Rotate during movement only.
        if action
            .clamped_axis_pair(&PlayerAction::Move)
            .length_squared()
            <= f32::EPSILON
        {
            continue;
        }

        let Some(direction) =
            Vec2::new(linear_velocity.x, linear_velocity.z)
                .try_normalize()
        else {
            continue;
        };

        let target_rotation = Quat::from_rotation_y(f32::atan2(
            -direction.x,
            -direction.y,
        ));

        rotation.0 =
            rotation.0.slerp(target_rotation, dt * ROTATION_RATE);
    }
}

/// Applies gravity to vertical velocity
fn apply_gravity(
    mut q_characters: Query<(
        &mut LinearVelocity,
        &CharacterController,
        &IsGrounded,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut linear_velocity, character, is_grounded) in
        q_characters.iter_mut()
    {
        if is_grounded.0 == false {
            linear_velocity.0 += character.gravity * dt;
        }
    }
}

/// Handles movement and jumping
fn movement(
    time: Res<Time>,
    q_camera_transform: Query<&GlobalTransform, With<Camera3d>>,
    q_actions: Query<&ActionState<PlayerAction>>,
    mut q_characters: Query<(
        &CharacterController,
        &mut LinearVelocity,
        &TargetAction,
        &PlayerType,
    )>,
) {
    let dt = time.delta_secs_f64() as f32;

    // Get camera transform.
    let Ok(cam_global_transform) = q_camera_transform.single() else {
        return;
    };

    let cam_forward = cam_global_transform.forward();
    let cam_forward =
        Vec2::new(cam_forward.x, cam_forward.z).normalize_or_zero();
    let cam_left = cam_global_transform.left();
    let cam_left =
        Vec2::new(cam_left.x, cam_left.z).normalize_or_zero();

    for (
        character,
        mut linear_velocity,
        target_action,
        player_type,
    ) in q_characters.iter_mut()
    {
        let Ok(action) = q_actions.get(target_action.get()) else {
            warn!("No `InputMap` found for player: {player_type:?}");
            continue;
        };

        let movement = action
            .clamped_axis_pair(&PlayerAction::Move)
            .clamp_length_max(1.0);
        if movement.length_squared() <= f32::EPSILON {
            // Ignore movement when it's negligible.
            continue;
        }

        // Compute yaw directly from that vector: atan2(x, z)
        let world_move = (cam_forward * movement.y)
            - (cam_left * movement.x).normalize_or_zero();
        let world_move = Vec3::new(world_move.x, 0.0, world_move.y);

        // Compute yaw and apply offset based on model orientation
        // let yaw = world_move.y.atan2(world_move.x);

        // Rotate to face movement direction
        // transform.rotation = Quat::from_rotation_y(yaw);

        // Only allow sprinting if grounded
        // let can_sprint = *sprint && is_grounded.0;
        let is_sprinting = false;

        // Apply acceleration * sprint factor
        let factor = if is_sprinting { 2.0 } else { 1.0 };
        let acceleration = character.acceleration;
        linear_velocity.0 +=
            world_move * (acceleration * dt * factor);

        // Clamp horizontal speed (only sprint speed if grounded)
        let max_speed = match is_sprinting {
            true => character.max_sprint,
            false => character.max_walk,
        };

        let horiz =
            Vec2::new(linear_velocity.0.x, linear_velocity.0.z);
        if horiz.length() > max_speed {
            let clamped = horiz.normalize() * max_speed;
            linear_velocity.0.x = clamped.x;
            linear_velocity.0.z = clamped.y;
        }
    }
}

/// Applies damping to horizontal movement
fn movement_damping(
    mut q_characters: Query<(
        &mut LinearVelocity,
        &CharacterController,
    )>,
) {
    for (mut linear_velocity, character) in q_characters.iter_mut() {
        // Damping cannot go above 1.0.
        let damping = character.damping.min(1.0);
        // Apply damping directly to physics velocity, except gravity.
        linear_velocity.x *= damping;
        linear_velocity.z *= damping;
    }
}

/// Handles collisions for kinematic character controllers
fn kinematic_controller_collisions(
    collisions: Collisions,
    bodies: Query<&RigidBody>,
    collider_rbs: Query<&ColliderOf, Without<Sensor>>,
    mut q_characters: Query<
        (
            &mut Position,
            &mut LinearVelocity,
            &CharacterController,
            &mut IsGrounded,
        ),
        (With<RigidBody>, With<CharacterController>),
    >,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for contacts in collisions.iter() {
        // Pull out the two bodies
        let Ok([&ColliderOf { body: a }, &ColliderOf { body: b }]) =
            collider_rbs
                .get_many([contacts.collider1, contacts.collider2])
        else {
            continue;
        };

        // Figure out which one is me
        let (entity, is_first, other) = if q_characters.get(a).is_ok()
        {
            (a, true, b)
        } else if q_characters.get(b).is_ok() {
            (b, false, a)
        } else {
            continue;
        };

        // Only do kinematic
        if !bodies.get(entity).unwrap().is_kinematic() {
            continue;
        }

        let (mut pos, mut linear_velocity, ctl, mut is_grounded) =
            q_characters.get_mut(entity).unwrap();

        // Detect if the other collider is dynamic
        let other_dynamic =
            bodies.get(other).is_ok_and(|rb| rb.is_dynamic());

        for manifold in &contacts.manifolds {
            let normal = if is_first {
                -manifold.normal
            } else {
                manifold.normal
            };

            // Push out of penetration and handle velocity
            let mut deepest = 0.0;
            for pt in &manifold.points {
                if pt.penetration > 0.0 {
                    let is_ground = normal.y > 0.7;
                    let is_jumping = linear_velocity.y > 0.0;

                    // Apply penetration correction unless jumping into ceiling
                    if !(is_ground && is_jumping) {
                        pos.0 += normal * pt.penetration;
                    }

                    // Cancel all vertical velocity when grounded
                    if is_ground {
                        linear_velocity.y = 0.0;
                        is_grounded.0 = true;
                    }
                }
                deepest = f32::max(deepest, pt.penetration);
            }

            // Skip dynamic collisions
            if other_dynamic {
                continue;
            }

            let slope_angle = normal.angle_between(Vec3::Y).abs();
            let can_climb = slope_angle <= ctl.max_slope_angle;

            if deepest > 0.0 {
                if can_climb {
                    // slope-snap logic
                    let dir_xz = normal
                        .reject_from_normalized(Vec3::Y)
                        .normalize_or_zero();
                    let vel_xz = linear_velocity.dot(dir_xz);
                    let max_y = -vel_xz * slope_angle.tan();
                    linear_velocity.y = linear_velocity.y.max(max_y);
                } else {
                    // Wall-slide: zero out velocity into the wall
                    let into = linear_velocity.dot(normal);
                    if into < 0.0 {
                        linear_velocity.0 -= normal * into;
                    }
                }
            } else {
                // Speculative contact
                let n_speed = linear_velocity.dot(normal);
                if n_speed < 0.0 {
                    let impulse = (n_speed - (deepest / dt)) * normal;
                    if can_climb {
                        linear_velocity.y -= impulse.y.min(0.0);
                    } else {
                        let mut i = impulse;
                        i.y = i.y.max(0.0);
                        linear_velocity.0 -= i;
                    }
                }
            }
        }
    }
}

/// Marker for kinematic character bodies
#[derive(Component, Reflect)]
#[require(IsGrounded, RequireAction)]
#[reflect(Component)]
pub struct CharacterController {
    /// Acceleration applied during moveme movement.
    pub acceleration: f32,
    /// Maximum velocity of walking.
    pub max_walk: f32,
    /// Maximum velocity of sprinting.
    pub max_sprint: f32,
    /// Damping value applied every frame (should be below 1.0).
    pub damping: f32,
    pub jump_impulse: f32,
    pub max_slope_angle: f32,
    pub gravity: Vec3,
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct IsGrounded(pub bool);
